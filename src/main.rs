#![windows_subsystem = "windows"]

mod configs;
mod tournament_info;

use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::{io, process};
use std::io::Write;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use eframe::CreationContext;
use egui_extras::{Column, TableBuilder};
use egui::{TextWrapMode, Ui, Visuals};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde_json::Map;

use configs::{get_config, get_config_dir, get_config_file, read_athletes, read_club,
    translate, write_athletes, write_club, write_config, write_tournaments};
use tournament_info::{registering_athletes_to_tournaments, Athlete, Belt,
    Club, GenderCategory, RegisteringAthlete, WeightCategory};

#[cfg(not(feature = "unstable"))]
static DEFAULT_TRANSLATIONS_DE: &str = include_str!("../lang/de.json");
#[cfg(not(feature = "unstable"))]
static DEFAULT_TRANSLATIONS_EN: &str = include_str!("../lang/en.json");

#[cfg(not(feature = "unstable"))]
static VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "unstable")]
static VERSION: &str = "unstable";
static LICENSE: &str = "GNU GPL v2";
static LICENSE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui/blob/master/LICENSE";
static CODE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui";

lazy_static::lazy_static! {
    static ref LANG_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("de", "Deutsch");
        m.insert("en", "English");
        m
    };
}

fn get_default_config() -> io::Result<(String, PathBuf)> {
    let athletes_file = get_config_dir()?.join("e-melder").join("athletes.json");
    let club_file = get_config_dir()?.join("e-melder").join("club.json");
    let tournament_basedir = home::home_dir().ok_or(io::Error::other("users does not have a home-directory"))?.join("e-melder");
    let mut default_config = Map::new();
    default_config.insert(String::from("lang"), "de".into());
    default_config.insert(String::from("dark-mode"), false.into());
    default_config.insert(String::from("club-file"), club_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("athletes-file"), athletes_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("tournament-basedir"), tournament_basedir.to_str().expect("unreachable").into());
    Ok((serde_json::to_string(&default_config).expect("unreachable"), tournament_basedir))
}

#[derive(Debug)]
enum UpdateAvailability {
    UpdateAvailable,
    NoUpdateAvailable,
    RunningUnstable
}

impl From<bool> for UpdateAvailability {
    fn from(value: bool) -> Self {
        if value { Self::UpdateAvailable }
        else { Self::NoUpdateAvailable }
    }
}

fn check_update_available(current_version: &str) -> io::Result<UpdateAvailability> {
    if current_version == "unstable" {
        return Ok(UpdateAvailability::RunningUnstable);
    }
    let body = reqwest::blocking::Client::builder().user_agent("").build().map_err(|err| {
        io::Error::other(err)
    })?.get("https://api.github.com/repos/UchiWerfer/e-melder-gui/releases/latest").send().map_err(|err| {
        io::Error::other(err)
    })?.text().map_err(|err| {
        io::Error::other(err)
    })?;
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    let version_value = parsed.get("tag_name").ok_or(io::Error::other("did not get \"tag_name\" attribute in api-response"))?;
    let version = version_value.as_str().ok_or(io::Error::other("\"tag_name\" attribute is not a string"))?;
    Ok(((String::from("v") + current_version) != version).into())
}

#[derive(Default, Debug)]
enum Mode {
    #[default]
    Registering,
    Adding,
    Deleting,
    Graduating,
    EditClub,
    Config,
    About
}

#[derive(Debug)]
struct Registering {
    athletes: Vec<RegisteringAthlete>,
    name: String,
    place: String,
    date: NaiveDate,
    search: String
}

impl Default for Registering {
    fn default() -> Self {
        Self {
            athletes: Vec::new(), name: String::new(), place: String::new(),
            date: Local::now().date_naive(), search: String::new()
        }
    }
}

#[derive(Debug)]
struct Adding {
    given_name: String,
    sur_name: String,
    belt: Belt,
    year: u16
}

impl Default for Adding {
    fn default() -> Self {
        Self {
            given_name: String::new(),
            sur_name: String::new(),
            belt: Belt::default(),
            year: 2010
        }
    }
}

impl Adding {
    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug)]
struct Config {
    lang: String,
    dark_mode: bool,
    athletes_file: PathBuf,
    club_file: PathBuf,
    tournament_basedir: String,
    langs: Vec<String>
}

#[derive(Debug)]
struct EMelderApp {
    athletes: Vec<Athlete>,
    club: Club,
    registering: Registering,
    adding: Adding,
    mode: Mode,
    config: Config,
    update_check_text: Option<String>,
    popup_open: bool
}

impl EMelderApp {
    fn new(cc: &CreationContext) -> io::Result<Self> {
        let athlete_file_value = get_config("athletes-file")?;
        let club_file_value = get_config("club-file")?;
        let dark_mode_value = get_config("dark-mode")?;
        let athletes_file = PathBuf::from(athlete_file_value.as_str()
            .ok_or(io::Error::new(io::ErrorKind::Other, "athletes-file not a string"))?);
        let club_file = PathBuf::from(club_file_value.as_str()
            .ok_or(io::Error::new(io::ErrorKind::Other, "club-file not a string"))?);
        let dark_mode = dark_mode_value.as_bool().ok_or(io::Error::new(
            io::ErrorKind::Other, "dark-mode not a bool"))?;
        let athletes = match read_athletes(&athletes_file) {
            Ok(athletes) => athletes,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    // e.g. at initial run or for using an alternative athletes-file
                    Vec::new()
                }
                else {
                    log::warn!("failed to read athletes, due to {err}");
                    Vec::new()
                }
            }
        };
        let club = match read_club(&club_file) {
            Ok(club) => club,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    // e.g. at initial run or for using an alternative club-file
                    Club::default()
                }
                else {
                    log::warn!("failed to read club, due to {err}");
                    Club::default()
                }
            }
        };
        let languages = std::fs::read_dir(get_config_dir()?.join("e-melder").join("lang"))?.map(|entry| {
            entry.unwrap_or_else(|err| {
                log::error!("failed to read config-directory/e-melder/lang, due to {err}");
                crash();
            }).path().file_stem().expect("unreachable").to_str().expect("unreachable").to_owned()
        }).collect();

        let visuals = if dark_mode { Visuals::dark() } else { Visuals::light() };
        
        cc.egui_ctx.set_visuals(visuals);
        Ok(Self {
            athletes, club, registering: Registering::default(), adding: Adding::default(), mode: Mode::default(),
            #[allow(clippy::single_match_else)]
            config: Config {
                lang: match get_config("lang") {
                    Ok(value) => match value.as_str() {
                        Some(lang) => lang,
                        None => {
                            log::error!("lang-config is not a string");
                            crash();
                        }
                    }.to_owned(),
                    Err(err) => {
                        log::error!("could not get lang-config, due to {err}");
                        crash();
                    }
                }, dark_mode, athletes_file,
                club_file,
                tournament_basedir: match get_config("tournament-basedir") {
                    Ok(value) => match value.as_str() {
                        Some(tournament_basedir) => tournament_basedir,
                        None => {
                            log::error!("tournament-basedir-config is not a string");
                            crash();
                        }
                    }.to_owned(),
                    Err (err) => {
                        log::error!("could net get tournament-basedir-config, due to {err}");
                        crash();
                    }
                },
                langs: languages
            }, popup_open: false, update_check_text: None
        })
    }

    #[allow(clippy::too_many_lines)]
    fn show_registering(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("register.name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("register.name")
                }
            });
            ui.text_edit_singleline(&mut self.registering.name);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.place") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("register.place")
                }
            });
            ui.text_edit_singleline(&mut self.registering.place);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.date") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("register.date")
                }
            });
            ui.add(egui_extras::DatePickerButton::new(&mut self.registering.date).format("%d.%m.%Y"));
        });

        if ui.button(match translate("register.register") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("register.register")
            }
        }).clicked() {
            let tournaments = registering_athletes_to_tournaments(
                &self.registering.athletes, &self.registering.name, self.registering.date,
                &self.registering.place, &self.club);
            
            let written = if let Some(tournaments) = tournaments {
                match write_tournaments(&tournaments) {
                    Ok(()) => {
                        true
                    }
                    Err(err) => {
                        log::warn!("failed to write tournaments, due to {err}");
                        false
                    }
                }
            } else { false };

            if written {
                #[allow(clippy::single_match_else)]
                let tournament_basedir = match get_config("tournament-basedir") {
                    Ok(tournament_basedir) => match tournament_basedir.as_str() {
                        Some(tournament_basedir) => PathBuf::from(tournament_basedir),
                        None => {
                            log::error!("tournament-basedir-config is not a string");
                            crash()
                        }
                    },
                    Err(err) => {
                        log::error!("failed to get tournament-basedir-config, due to {err}");
                        crash()
                    }
                };

                #[cfg(all(target_family="unix", not(target_os="macos")))]
                std::thread::spawn(|| {
                    let _ = notify_rust::Notification::new()
                    .summary(&match translate("application.title") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("application.title")
                        }
                    })
                    .body(&match translate("register.notification.ask") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.applicatoin.ask")
                        }
                    })
                    .sound_name("dialog-question")
                    .action("yes", &match translate("register.notification.yes") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.notification.yes")
                        }
                    })
                    .action("no", &match translate("register.notification.no") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.notification.no")
                        }
                    })
                    .show().map(|handle| {
                        handle.wait_for_action(|action| {
                            if action == "yes" {
                                let _ = open::that_detached(tournament_basedir);
                            }
                        });
                    });
                });

                #[cfg(any(not(target_family="unix"), target_os="macos"))]
                let _ = open::that_detached(tournament_basedir);
            }
        }

        ui.separator();

        self.show_table_registering_adding(ui);

        ui.separator();

        self.show_table_registering(ui);
    }

    fn show_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("add.given_name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("add.given_name")
                }
            });
            ui.text_edit_singleline(&mut self.adding.given_name);
        });
        ui.horizontal(|ui| {
            ui.label(match translate("add.sur_name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("add.sur_name")
                }
            });
            ui.text_edit_singleline(&mut self.adding.sur_name);
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(match translate("add.belt") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("add.belt")
                }
            }).selected_text(match translate(&format!("add.belt.{}", self.adding.belt.serialise())) {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    format!("add.belt.{}", self.adding.belt.serialise())
                }
            }).show_ui(ui, |ui| {
                for belt in [Belt::Kyu9, Belt::Kyu8, Belt::Kyu7, Belt::Kyu6, Belt::Kyu5, Belt::Kyu4, Belt::Kyu3, Belt::Kyu2, Belt::Kyu1,
                Belt::Dan1, Belt::Dan2, Belt::Dan3, Belt::Dan4, Belt::Dan5, Belt::Dan6, Belt::Dan7, Belt::Dan8, Belt::Dan9, Belt::Dan10] {
                    ui.selectable_value(&mut self.adding.belt, belt, match translate(
                        &format!("add.belt.{}", belt.serialise())) {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            format!("add.belt.{}", belt.serialise())
                        }
                    });
                }
            })
        });
        ui.horizontal(|ui| {
            ui.label(match translate("add.year") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("add.year")
                }
            });
            ui.add(egui::Slider::new(&mut self.adding.year, 1900..=2100));
        });

        if ui.button(match translate("add.commit") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("add.commit")
            }
        }).clicked() {
            self.athletes.push(Athlete::new(
                self.adding.given_name.clone(), self.adding.sur_name.clone(),
                self.adding.year, self.adding.belt, WeightCategory::default()
            ));
            self.adding.clear();
            let athletes_path = match get_config("athletes-file") {
                Ok(path) => path,
                Err(err) => {
                    log::error!("failed to get athletes-file-config, due to {err}");
                    crash();
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    log::error!("athletes-file-config is not a string");
                    crash();
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to write athletes, due to {err}");
                    crash();
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_graduating(&mut self, ui: &mut Ui) {
        if self.athletes.is_empty() {
            if ui.button(match translate("graduate.empty") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("graduate.empty")
                }
            }).clicked() {
                self.mode = Mode::Adding;
            }
            return;
        }


        let mut to_graduate = None;
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 4).column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(match translate("graduate.given_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("graduate.given_name")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.sur_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("graduate.sur_name")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.year") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("graduate.year")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.belt") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("graduate.belt")
                    }
                });
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in self.athletes.iter().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                format!("add.belt.{}", athlete.get_belt().serialise())
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(match translate("graduate.graduate") {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                String::from("graduate.graduate")
                            }
                        }).clicked() {
                            to_graduate = Some(index);
                        }
                    });
                });
            }
        });

        if let Some(index) = to_graduate {
            let belt = self.athletes[index].get_belt();
            *self.athletes[index].get_belt_mut() = belt.inc();
            let athletes_path = match get_config("athletes-file") {
                Ok(path) => path,
                Err(err) => {
                    log::error!("failed to get athletes-file-config, due to {err}");
                    crash();
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    log::error!("athletes-file not a string");
                    crash();
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to write athletes, due to {err}");
                    crash();
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_edit(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("edit.club_name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.club_name")
                }
            });
            ui.text_edit_singleline(self.club.get_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.given_name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.given_name")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_given_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.sur_name") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.sur_name")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_sur_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.address") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.address")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_address_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.postal_code") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.postal_code")
                }
            });
            ui.add(egui::DragValue::new(self.club.get_sender_mut().get_postal_code_mut()));
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.town") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.town")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_town_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.private") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.private")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_private_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.public") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.public")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_public_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.fax") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.fax")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_fax_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.mobile") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get transation, due to {err}");
                    String::from("edit.mobile")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_mobile_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.mail") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.mail")
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_mail_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.club_number") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.club_number")
                }
            });
            ui.add(egui::DragValue::new(self.club.get_number_mut()));
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.county") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.county")
                }
            });
            ui.text_edit_singleline(self.club.get_county_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.region") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.region")
                }
            });
            ui.text_edit_singleline(self.club.get_region_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.state") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.state")
                }
            });
            ui.text_edit_singleline(self.club.get_state_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.group") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.group")
                }
            });
            ui.text_edit_singleline(self.club.get_group_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.nation") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("edit.nation")
                }
            });
            ui.text_edit_singleline(self.club.get_nation_mut());
        });

        if ui.button(match translate("edit.save") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("edit.save")
            }
        }).clicked() {
            let path_value = match get_config("club-file") {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("failed to get config: {err}");
                    process::exit(1)
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match path_value.as_str() {
                Some(value) => value,
                None => {
                    log::error!("club-file-config is not a string");
                    crash();
                }
            });
            match write_club(path, &self.club) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to write club, due to {err}");
                    crash();
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_delete(&mut self, ui: &mut Ui) {
        if self.athletes.is_empty() {
            ui.label(match translate("delete.empty") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("delete.empty")
                }
            });
            return;
        }

        let mut to_delete = None;
        let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(match translate("delete.given_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("delete.given_name")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.sur_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("delete.sur_name")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.year") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("delete.year")
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.belt") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("delete.belt")
                    }
                });
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in self.athletes.iter().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                format!("add.belt.{}", athlete.get_belt().serialise())
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(match translate("delete.delete") {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                String::from("delete.delete")
                            }
                        }).clicked() {
                            to_delete = Some(index);
                        }
                    });
                });
            }
        });

        if let Some(index) = to_delete {
            self.athletes.remove(index);
            let athletes_path = match get_config("athletes-file") {
                Ok(path) => path,
                Err(err) => {
                    log::error!("failed to get athletes-file-config, due to {err}");
                    crash();
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    log::error!("athletes-file-config is not a string");
                    crash();
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to write athletes, due to {err}");
                    crash();
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_config(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(match translate("config.lang") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("config.lang")
                }
            })
            .selected_text(*LANG_NAMES.get(self.config.lang.as_str()).unwrap_or(&self.config.lang.as_str()))
            .show_ui(ui, |ui| {
                for lang in &self.config.langs {
                    ui.selectable_value(&mut self.config.lang, lang.clone(), *LANG_NAMES.get(lang.as_str()).unwrap_or(&lang.as_str()));
                }
            });
        });
        
        ui.checkbox(&mut self.config.dark_mode, match translate("config.dark_mode") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("config.dark_mode")
            }
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.select_athletes_file") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("config.select_athletes_file")
                }
            });
            if ui.button(self.config.athletes_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(match translate("config.athletes_file.file_picker") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("config.athletes_file.file_picker")
                        }
                    }).save_file() {
                        Some(athletes_file) => {
                            self.config.athletes_file = athletes_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.select_club_file") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("config.select_club_file")
                }
            });
            if ui.button(self.config.club_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(match translate("config.club_file.file_picker") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation: {err}");
                            String::from("config.club_file.file_picker")
                        }
                    }).save_file() {
                        Some(club_file) => {
                            self.config.club_file = club_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.select_tournament_basedir") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation: {err}");
                    String::from("config.select_tournament_basedir")
                }
            });
            if ui.button(&self.config.tournament_basedir).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_directory(&self.config.tournament_basedir)
                    .set_can_create_directories(true).set_title(match translate("config.tournament_basedir.file_picker") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation: {err}");
                            String::from("conig.tournament_basedir.file_picker")
                        }
                    }).pick_folder() {
                        Some(directory) => {
                            self.config.tournament_basedir = directory.display().to_string();
                        },
                        None => {}
                    }
            }
        });

        if ui.button(match translate("config.save") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("config.save")
            }
        }).clicked() {
            match write_config("lang", self.config.lang.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to set config, due to {err}");
                    crash();
                }
            }
            
            match write_config("dark-mode", self.config.dark_mode.into()) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to set config, due to {err}");
                    crash();
                }
            }

            match write_config("athletes-file", self.config.athletes_file.display().to_string().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to set config, due to {err}");
                    crash();
                }
            }

            match write_config("club-file", self.config.club_file.display().to_string().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to set config, due to {err}");
                    crash();
                }
            }

            match write_config("tournament-basedir", self.config.tournament_basedir.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::error!("failed to set config, due to {err}");
                    crash();
                }
            }
        }
    }

    fn show_about(&mut self, ui: &mut Ui) {
        ui.label(match translate("about.about") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("about.about")
            }
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(match translate("about.version") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("about.version")
                }
            });
            ui.label(VERSION);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("about.license") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("about.license")
                }
            });
            if ui.link(LICENSE).on_hover_text(LICENSE_LINK).clicked() {
                let _ = open::that_detached(LICENSE_LINK);
            }
        });

        ui.horizontal(|ui| {
            ui.label(match translate("about.source_code") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("about.source_code")
                }
            });
            if ui.link(CODE_LINK).on_hover_text(CODE_LINK).clicked() {
                let _ = open::that_detached(CODE_LINK);
            }
        });

        if ui.button(match translate("about.check_update") {
            Ok(translation) => translation,
            Err(err) => {
                log::warn!("failed to get translation, due to {err}");
                String::from("about.check_update")
            }
        }).clicked() {
            let update_available = check_update_available(VERSION);
            self.popup_open = true;
            if let Ok(update_available) = update_available {
                match update_available {
                    UpdateAvailability::UpdateAvailable => {
                        self.update_check_text = Some(match translate("about.update_available") {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                String::from("about.update_available")
                            }
                        });
                    }
                    UpdateAvailability::NoUpdateAvailable => {
                        self.update_check_text = Some(match translate("about.no_update_available") {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                String::from("about.no_update_available")
                            }
                        });
                    }
                    UpdateAvailability::RunningUnstable => {
                        self.update_check_text = Some(match translate("about.running_unstable") {
                            Ok(translation) => translation,
                            Err(err) => {
                                log::warn!("failed to get translation, due to {err}");
                                String::from("about.running_unstable")
                            }
                        });
                    }
                }
            }
            else {
                log::warn!("failed to get new version information from network: {}", update_available.unwrap_err()); // cannot panic, as it was checked above for `Ok`
                self.update_check_text = Some(match translate("about.no_network") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("about.no_network")
                    }
                });
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_table_registering(&mut self, ui: &mut Ui) {
        let mut to_delete = None;
        ui.push_id("register.table.register", |ui| {
            let table = TableBuilder::new(ui)
                .columns(Column::auto().at_least(100.0), 7)
                .column(Column::auto().at_least(50.0));

            table.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(match translate("register.table.given_name") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.given_name")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.sur_name") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.sur_name")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.belt") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.belt")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.year") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.year")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.gender_category") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.gender_category")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.age_category") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.age_category")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.weight_category") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.weight_category")
                        }
                    });
                });
                header.col(|_ui| {});
            }).body(|mut body| {
                for (index, athlete) in self.registering.athletes.iter_mut().enumerate() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(athlete.get_given_name());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(athlete.get_sur_name());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                                Ok(translation) => translation,
                                Err(err) => {
                                    log::warn!("failed to get translation, due to {err}");
                                    format!("add.belt.{}", athlete.get_belt().serialise())
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.label(athlete.get_birth_year().to_string());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            egui::ComboBox::from_id_source(index)
                            .selected_text(match translate(
                                &format!("register.table.gender_category.{}", athlete.get_gender_category().render())) {
                                    Ok(translation) => translation,
                                    Err(err) => {
                                        log::warn!("failed to get translation, due to {err}");
                                        format!("register.table.gender_category.{}", athlete.get_gender_category().render())
                                    }
                                }).show_ui(ui, |ui| {
                                    for gender_category in [GenderCategory::Mixed, GenderCategory::Female, GenderCategory::Male] {
                                        ui.selectable_value(athlete.get_gender_category_mut(), gender_category,
                                            match translate(&format!("register.table.gender_category.{}", gender_category.render())) {
                                                Ok(translation) => translation,
                                                Err(err) => {
                                                    log::warn!("failed to get translation: {err}");
                                                    format!("register.table.gender_category.{}", gender_category.render())
                                                }
                                            });
                                    }
                                });
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(athlete.get_age_category_mut());
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(athlete.get_weight_category_mut());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            if ui.button(match translate("register.table.delete") {
                                Ok(translation) => translation,
                                Err(err) => {
                                    log::warn!("failed to get translation, due to {err}");
                                    String::from("register.table.delete")
                                }
                            }).clicked() {
                                to_delete = Some(index);
                            }
                        });
                    });
                }
            });
        });

        if let Some(index) = to_delete {
            self.registering.athletes.remove(index);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_table_registering_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("register.search") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to obtain translation, due to {err}");
                    String::from("register.search")
                }
            });
            ui.text_edit_singleline(&mut self.registering.search);
        });

        let mut athletes_shown = false;
        ui.push_id("register.table.add", |ui| {
            let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
                .column(Column::auto().at_least(50.0)).max_scroll_height(100.0);

            table.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(match translate("register.table.given_name") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.given_name")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.sur_name") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.sur_name")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.belt") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.belt")
                        }
                    });
                });
                header.col(|ui| {
                    ui.strong(match translate("register.table.year") {
                        Ok(translation) => translation,
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            String::from("register.table.year")
                        }
                    });
                });
            }).body(|mut body| {
                for athlete in &self.athletes {
                    if !format!("{} {}", athlete.get_given_name(), athlete.get_sur_name()).contains(&self.registering.search) {
                        continue;
                    }
                    athletes_shown = true;

                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(athlete.get_given_name());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(athlete.get_sur_name());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                                Ok(translation) => translation,
                                Err(err) => {
                                    log::warn!("failed to get translation, due to {err}");
                                    format!("add.belt.{}", athlete.get_belt().serialise())
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.label(athlete.get_birth_year().to_string());
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            if ui.button(match translate("register.table.add") {
                                Ok(translation) => translation,
                                Err(err) => {
                                    log::warn!("failed to get translation, due to {err}");
                                    String::from("register.table.add")
                                }
                            }).clicked() {
                                self.registering.athletes.push(RegisteringAthlete::from_athlete(athlete));
                            }
                        });
                    });
                }
            });
        });

        if !athletes_shown {
            ui.label(match translate("register.empty") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("register.empty")
                }
            });
        }
    }
}

impl eframe::App for EMelderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.popup_open && self.update_check_text.is_some() {
            self.update_check_text = None;
        }

        if let Some(update_check_text) = &self.update_check_text {
            egui::Window::new(match translate("about.update_popup_title") {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    String::from("about.update_popup_title")
                }
            }).collapsible(false).resizable(false).open(&mut self.popup_open).show(ctx, |ui| {
                ui.label(update_check_text);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.update_check_text.is_some() {
                ui.disable();
            }
            egui::menu::bar(ui, |ui| {
                if ui.button(match translate("application.register") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.register")
                    }
                }).clicked() {
                    self.mode = Mode::Registering;
                }

                if ui.button(match translate("application.add") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.add")
                    }
                }).clicked() {
                    self.mode = Mode::Adding;
                }

                if ui.button(match translate("application.graduate") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.graduate")
                    }
                }).clicked() {
                    self.mode = Mode::Graduating;
                }

                if ui.button(match translate("application.delete") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.delete")
                    }
                }).clicked() {
                    self.mode = Mode::Deleting;
                }

                if ui.button(match translate("application.edit") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.edit")
                    }
                }).clicked() {
                    self.mode = Mode::EditClub;
                }

                if ui.button(match translate("application.config") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.config")
                    }
                }).clicked() {
                    self.mode = Mode::Config;
                }

                if ui.button(match translate("application.about") {
                    Ok(translation) => translation,
                    Err(err) => {
                        log::warn!("failed to get translation, due to {err}");
                        String::from("application.about")
                    }
                }).clicked() {
                    self.mode = Mode::About;
                }
            });

            match self.mode {
                Mode::Registering => self.show_registering(ui),
                Mode::Adding => self.show_adding(ui),
                Mode::Graduating => self.show_graduating(ui),
                Mode::EditClub => self.show_edit(ui),
                Mode::Deleting => self.show_delete(ui),
                Mode::Config => self.show_config(ui),
                Mode::About => self.show_about(ui)
            }
            #[cfg(feature="debugging")]
            if ui.button("debug").clicked() {
                dbg!(self);
            }
        });
    }
}

fn crash() -> ! {
    let _ = notify_rust::Notification::new()
    .summary("E-Melder")
    .body(&format!("An unrecoverable error occurred, please look into the logs to see what happened.\n{}{}",
    "If you think this is a bug, please file a bug report at ", CODE_LINK))
    .sound_name("dialog-error")
    .show();
    panic!()
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), eframe::Error> {
    let stdout_logger = ConsoleAppender::builder().build();
    let file_logger = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{level} from {module} on {date(%a, %Y-%m-%d at %H:%M:%S%z)}: {message}\n")))
        .build(get_config_dir().unwrap_or_else(|_err| {
            crash()
        }).join("e-melder/e-melder.log")).unwrap_or_else(
            |_err| {
                crash()
            }
        );
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout_logger)))
        .appender(Appender::builder().build("file", Box::new(file_logger)))
        .logger(Logger::builder()
            .appenders(["stdout", "file"])
            .build("e-melder", log::LevelFilter::Info))
        .build(Root::builder().appenders(["stdout", "file"]).build(log::LevelFilter::Info)).unwrap_or_else(|_err| {
            crash()
        });
    log4rs::init_config(config).unwrap_or_else(|_err| {
        crash()
    });
    log::info!("New run of the app");

    let config_file = match get_config_file() {
        Ok(config_file) => config_file,
        Err(err) => {
            log::error!("failed to get config-file, due to {err}");
            crash();
        }
    };

    if !config_file.exists() {
        if let Some(config_file_parent) = config_file.parent() {
            match create_dir_all(config_file_parent) {
                Ok(()) => {}
                Err(err) => {
                    log::error!("failed to create neccessary directories for config-file, due to {err}");
                    crash();
                }
            }
        }

        let mut config_file = match File::options().write(true).create_new(true).open(config_file) {
            Ok(config_file) => config_file,
            Err(err) => {
                log::error!("failed to create config-file, due to {err}");
                crash();
            }
        };

        let (default_configs, tournament_basedir) = match get_default_config() {
            Ok(default_configs) => default_configs,
            Err(err) => {
                log::error!("failed to create default-configs, due to {err}");
                crash();
            }
        };

        match config_file.write_all(default_configs.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                log::warn!("failed to write default-configs, due to {err}");
            }
        }

        match create_dir_all(tournament_basedir) {
            Ok(()) => {},
            Err(err) => {
                log::warn!("failed to create neccessary directories for tournament-basedir, due to {err}");
            }
        }
    }

    #[cfg(not(feature = "unstable"))]
    let lang_file = match get_config_dir() {
        Ok(lang_file) => lang_file,
        Err(err) => {
            log::error!("failed to get config dir, due to {err}");
            crash();
        }
    }.join("e-melder").join("lang").join(format!("{}.json", get_config("lang").unwrap_or_else(|err| {
        log::error!("failed to get language, due to {err}");
        crash();
    }).as_str().unwrap_or_else(|| {
        log::error!("language-config is not a string");
        crash();
    })));

    #[cfg(not(feature = "unstable"))]
    if !lang_file.exists() {
        match create_dir_all(lang_file.parent().expect("unreachable")) {
            Ok(()) => {},
            Err(err) => {
                log::error!("failed to create neccessary directories for lang-file, due to {err}");
                crash();
            }   
        }

        let mut lang_file = match File::options().write(true).create_new(true).open(lang_file) {
            Ok(lang_file) => lang_file,
            Err(err) => {
                log::error!("failed to create lang-file, due to {err}");
                crash();
            }
        };

        let lang_value = get_config("lang").unwrap_or_else(|err| {
            log::error!("failed to get lang-config, due to {err}");
            crash();
        });
        let lang = lang_value.as_str().unwrap_or_else(|| {
            log::error!("lang-config is not a string");
            crash();
        });
        let translations = match lang {
            "de" => DEFAULT_TRANSLATIONS_DE,
            "en" => DEFAULT_TRANSLATIONS_EN,
            // other languages, that might be supported in the future, would be listed here
            _ => "{}"
        };

        match lang_file.write_all(translations.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                log::error!("failed to write default language, due to {err}");
                crash();
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,

        ..Default::default()
    };

    eframe::run_native(match translate("application.title") {
        Ok(title) => title,
        Err(err) => {
            log::error!("failed to setup app, due to {err}");
            crash();
        }
    }.as_str(), options, Box::new(|cc| {
        match EMelderApp::new(cc) {
            Ok(app) => Ok(Box::new(app)),
            Err(err) => Err(Box::new(err))
        }
    }))
}
