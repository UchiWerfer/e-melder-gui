mod configs;
mod tournament_info;

use std::fs::{create_dir_all, File};
use std::{io, process};
use std::io::Write;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use eframe::CreationContext;
use egui_extras::{Column, TableBuilder};
use egui::{Ui, Visuals};
use serde_json::Map;

use configs::{get_config, get_config_dir, get_config_file, read_athletes, read_club, translate, write_athletes, write_club, write_config, write_tournaments};
use tournament_info::{registering_athletes_to_tournaments, Athlete, Belt, Club, GenderCategory, RegisteringAthlete, WeightCategory};

static DEFAULT_TRANSLATIONS_DE: &str = include_str!("../lang/de.json");
static DEFAULT_TRANSLATIONS_EN: &str = include_str!("../lang/en.json");

static VERSION: &str = env!("CARGO_PKG_VERSION");
static LICENSE: &str = "GNU GPL v2";
static LICENSE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui/blob/master/LICENSE";
static CODE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui";

fn get_default_config() -> io::Result<(String, PathBuf, PathBuf)> {
    let athletes_file = get_config_dir()?.join("e-melder").join("athletes.json");
    let club_file = get_config_dir()?.join("e-melder").join("club.json");
    let mut default_config = Map::new();
    default_config.insert(String::from("lang"), "de".into());
    default_config.insert(String::from("dark-mode"), false.into());
    default_config.insert(String::from("club-file"), club_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("athletes-file"), athletes_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("tournament-basedir"), "".into());
    Ok((serde_json::to_string(&default_config).expect("unreachable"), athletes_file, club_file))
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
    given_name: String,
    sur_name: String
}

impl Default for Registering {
    fn default() -> Self {
        Self {
            athletes: Vec::new(), name: String::new(), place: String::new(),
            date: Local::now().date_naive(), given_name: String::new(), sur_name: String::new()
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

#[derive(Debug)]
struct Config {
    lang: String,
    dark_mode: bool,
    athletes_file: String,
    club_file: String,
    tournament_basedir: String
}

#[derive(Debug)]
struct EMelderApp {
    athletes: Vec<Athlete>,
    club: Club,
    registering: Registering,
    adding: Adding,
    mode: Mode,
    config: Config
}

impl EMelderApp {
    fn new(cc: &CreationContext) -> io::Result<Self> {
        let athlete_file_value = get_config("athletes-file")?;
        let club_file_value = get_config("club-file")?;
        let dark_mode_value = get_config("dark-mode")?;
        let athlete_file = PathBuf::from(athlete_file_value.as_str()
            .ok_or(io::Error::new(io::ErrorKind::Other, "athletes-file not a string"))?);
        let club_file = PathBuf::from(club_file_value.as_str()
            .ok_or(io::Error::new(io::ErrorKind::Other, "club-file not a string"))?);
        let dark_mode = dark_mode_value.as_bool().ok_or(io::Error::new(
            io::ErrorKind::Other, "dark-mode not a bool"))?;
        let athletes = match read_athletes(athlete_file) {
            Ok(athletes) => athletes,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    // e.g. at initial run or for using an alternative athletes-file
                    Vec::new()
                }
                else {
                    eprintln!("failed to read athletes: {err}");
                    process::exit(1)
                }
            }
        };
        let club = match read_club(club_file) {
            Ok(club) => club,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    // e.g. at initial run or for using an alternative club-file
                    Club::default()
                }
                else {
                    eprintln!("failed to read club: {err}");
                    process::exit(1)
                }
            }
        };

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
                            eprintln!("lang not a string");
                            process::exit(1)
                        }
                    }.to_owned(),
                    Err(err) => {
                        eprintln!("failed to get config lang: {err}");
                        process::exit(1)
                    }
                }, dark_mode, athletes_file: athlete_file_value.as_str().expect("unreachable").to_owned(),
                club_file: club_file_value.as_str().expect("unreachable").to_owned(),
                tournament_basedir: match get_config("tournament-basedir") {
                    Ok(value) => match value.as_str() {
                        Some(tournament_basedir) => tournament_basedir,
                        None => {
                            eprintln!("tournament-basedir not a string");
                            process::exit(1)
                        }
                    }.to_owned(),
                    Err (err) => {
                        eprintln!("failed to get config tournament-basedir: {err}");
                        process::exit(1)
                    }
                }
            }
        })
    }

    fn show_registering(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("register.name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.registering.name);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.place") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.registering.place);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.date") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.add(egui_extras::DatePickerButton::new(&mut self.registering.date));
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.given_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.registering.given_name);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("register.sur_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.registering.sur_name);
        });

        if ui.button(match translate("register.add") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        }).clicked() {
            for athlete in &self.athletes {
                if athlete.get_sur_name() == self.registering.sur_name && athlete.get_given_name() == self.registering.given_name {
                    self.registering.athletes.push(
                        RegisteringAthlete::from_athlete(athlete)
                    );
                    self.registering.sur_name.clear();
                    self.registering.given_name.clear();
                }
            }
        }

        if ui.button(match translate("register.register") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        }).clicked() {
            let tournaments = registering_athletes_to_tournaments(
                &self.registering.athletes, &self.registering.name, self.registering.date,
                &self.registering.place, &self.club);
            #[allow(clippy::single_match_else)]
            match write_tournaments(&match tournaments {
                Some(tournaments) => tournaments,
                None => {
                    eprintln!("got invalid weight category");
                    vec![]
                }
            }) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to write tournaments: {err}");
                    process::exit(1);
                }
            }
        }

        ui.separator();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            self.show_table_registering(ui);
        });
    }

    fn show_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("add.given_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.adding.given_name);
        });
        ui.horizontal(|ui| {
            ui.label(match translate("add.sur_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.adding.sur_name);
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(match translate("add.belt") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            }).selected_text(match translate(&format!("add.belt.{}", self.adding.belt.serialise())) {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            }).show_ui(ui, |ui| {
                for belt in [Belt::Kyu9, Belt::Kyu8, Belt::Kyu7, Belt::Kyu6, Belt::Kyu5, Belt::Kyu4, Belt::Kyu3, Belt::Kyu2, Belt::Kyu1,
                Belt::Dan1, Belt::Dan2, Belt::Dan3, Belt::Dan4, Belt::Dan5, Belt::Dan6, Belt::Dan7, Belt::Dan8, Belt::Dan9, Belt::Dan10] {
                    ui.selectable_value(&mut self.adding.belt, belt, match translate(
                        &format!("add.belt.{}", belt.serialise())) {
                        Ok(translation) => translation,
                        Err(err) => {
                            eprintln!("failed to get translation: {err}");
                            process::exit(1)
                        }
                    });
                }
            })
        });
        ui.horizontal(|ui| {
            ui.label(match translate("add.year") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.add(egui::Slider::new(&mut self.adding.year, 1900..=2100));
        });

        if ui.button(match translate("add.commit") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        }).clicked() {
            self.athletes.push(Athlete::new(
                self.adding.given_name.clone(), self.adding.sur_name.clone(),
                self.adding.year, self.adding.belt, WeightCategory::default()
            ));
            let athletes_path = match get_config("athletes-file") {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("failed to get config: {err}");
                    process::exit(1)
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    eprintln!("athletes-file not a string");
                    process::exit(1)
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to write athletes: {err}");
                    process::exit(1)
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_graduating(&mut self, ui: &mut Ui) {
        let mut to_graduate = None;
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 4).column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(match translate("graduate.given_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.sur_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.year") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("graduate.belt") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in self.athletes.iter().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        if ui.button(match translate("graduate.graduate") {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
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
                    eprintln!("failed to get config: {err}");
                    process::exit(1)
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    eprintln!("athletes-file not a string");
                    process::exit(1)
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to write athletes: {err}");
                    process::exit(1);
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
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.given_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_given_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.sur_name") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_sur_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.address") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_address_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.postal_code") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.add(egui::DragValue::new(self.club.get_sender_mut().get_postal_code_mut()));
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.town") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_town_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(match translate("edit.private") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_private_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.public") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_public_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.fax") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_fax_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.mobile") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_mobile_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.mail") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_sender_mut().get_mail_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.club_number") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.add(egui::DragValue::new(self.club.get_number_mut()));
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.county") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_county_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.region") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_region_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.state") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_state_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.group") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_group_mut());
        });

        ui.horizontal(|ui| {
            ui.label(match translate("edit.nation") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(self.club.get_nation_mut());
        });

        if ui.button(match translate("edit.save") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
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
                    eprintln!("club-file not a string");
                    process::exit(1)
                }
            });
            match write_club(path, &self.club) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to write club: {err}");
                    process::exit(1);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn show_delete(&mut self, ui: &mut Ui) {
        let mut to_delete = None;
        let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(match translate("delete.given_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.sur_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.year") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("delete.belt") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in self.athletes.iter().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        if ui.button(match translate("delete.delete") {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
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
                    eprintln!("failed to get config: {err}");
                    process::exit(1)
                }
            };
            #[allow(clippy::single_match_else)]
            let path = PathBuf::from(match athletes_path.as_str() {
                Some(path) => path,
                None => {
                    eprintln!("athletes-file not a string");
                    process::exit(1)
                }
            });
            match write_athletes(path, &self.athletes) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to write athletes: {err}");
                    process::exit(1);
                }
            }
        }
    }

    fn show_config(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(match translate("config.lang") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.config.lang);
        });
        
        ui.checkbox(&mut self.config.dark_mode, match translate("config.dark_mode") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.athletes_file") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.config.athletes_file);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.club_file") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.config.club_file);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("config.tournament_basedir") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.text_edit_singleline(&mut self.config.tournament_basedir);
        });

        if ui.button(match translate("config.save") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        }).clicked() {
            match write_config("lang", self.config.lang.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to set config: {err}");
                    process::exit(1);
                }
            }
            
            match write_config("dark-mode", self.config.dark_mode.into()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to set config: {err}");
                    process::exit(1);
                }
            }

            match write_config("athletes-file", self.config.athletes_file.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to set config: {err}");
                    process::exit(1);
                }
            }

            match write_config("club-file", self.config.club_file.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to set config: {err}");
                    process::exit(1);
                }
            }

            match write_config("tournament-basedir", self.config.tournament_basedir.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("failed to set config: {err}");
                    process::exit(1);
                }
            }
        }
    }

    fn show_about(ui: &mut Ui) {
        ui.label(match translate("about.about") {
            Ok(translation) => translation,
            Err(err) => {
                eprintln!("failed to get translation: {err}");
                process::exit(1)
            }
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(match translate("about.version") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            ui.label(VERSION);
        });

        ui.horizontal(|ui| {
            ui.label(match translate("about.license") {
                Ok(translation) => translation,
                Err(err) => {
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
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
                    eprintln!("failed to get translation: {err}");
                    process::exit(1)
                }
            });
            if ui.link(CODE_LINK).on_hover_text(CODE_LINK).clicked() {
                let _ = open::that_detached(CODE_LINK);
            }
        });
    }

    #[allow(clippy::too_many_lines)]
    fn show_table_registering(&mut self, ui: &mut Ui) {
        let mut to_delete = None;
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 6)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(match translate("register.table.given_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.sur_name") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.belt") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.year") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.gender_category") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.age_category") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|ui| {
                ui.strong(match translate("register.table.weight_category") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                });
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in self.registering.athletes.iter_mut().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.label(match translate(&format!("add.belt.{}", athlete.get_belt().serialise())) {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        egui::ComboBox::from_id_source(index)
                        .selected_text(match translate(
                            &format!("register.table.gender_category.{}", athlete.get_gender_category().render())) {
                                Ok(translation) => translation,
                                Err(err) => {
                                    eprintln!("failed to get translation: {err}");
                                    process::exit(1)
                                }
                            }).show_ui(ui, |ui| {
                                for gender_category in [GenderCategory::Mixed, GenderCategory::Female, GenderCategory::Male] {
                                    ui.selectable_value(athlete.get_gender_category_mut(), gender_category,
                                        match translate(&format!("register.table.gender_category.{}", gender_category.render())) {
                                            Ok(translation) => translation,
                                            Err(err) => {
                                                eprintln!("failed to get translation: {err}");
                                                process::exit(1)
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
                        ui.style_mut().wrap = Some(false);
                        if ui.button(match translate("register.table.delete") {
                            Ok(translation) => translation,
                            Err(err) => {
                                eprintln!("failed to get translation: {err}");
                                process::exit(1)
                            }
                        }).clicked() {
                            to_delete = Some(index);
                        }
                    });
                });
            }
        });

        if let Some(index) = to_delete {
            self.registering.athletes.remove(index);
        }
    }
}

impl eframe::App for EMelderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button(match translate("application.register") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::Registering;
                }

                if ui.button(match translate("application.add") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::Adding;
                }

                if ui.button(match translate("application.graduate") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::Graduating;
                }

                if ui.button(match translate("application.delete") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::Deleting;
                }

                if ui.button(match translate("application.edit") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::EditClub;
                }

                if ui.button(match translate("application.config") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
                    }
                }).clicked() {
                    self.mode = Mode::Config;
                }

                if ui.button(match translate("application.about") {
                    Ok(translation) => translation,
                    Err(err) => {
                        eprintln!("failed to get translation: {err}");
                        process::exit(1)
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
                Mode::About => Self::show_about(ui)
            }
            #[cfg(feature="debugging")]
            if ui.button("debug").clicked() {
                dbg!(self);
            }
        });
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), eframe::Error> {
    let config_file = match get_config_file() {
        Ok(config_file) => config_file,
        Err(err) => {
            eprintln!("failed to get config-file: {err}");
            process::exit(1)
        }
    };

    if !config_file.exists() {
        #[allow(clippy::single_match_else)]
        match create_dir_all(match config_file.parent() {
            Some(config_file_parent) => config_file_parent,
            None => {
                eprintln!("config-file does not have a parent-directory");
                process::exit(1)
            }
        }) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to create neccessary directories for config-file: {err}");
                process::exit(1)
            }
        }

        let mut config_file = match File::options().write(true).create_new(true).open(config_file) {
            Ok(config_file) => config_file,
            Err(err) => {
                eprintln!("failed to create config file: {err}");
                process::exit(1)
            }
        };

        let (default_configs, athletes_file_path, club_file_path) = match get_default_config() {
            Ok(default_configs) => default_configs,
            Err(err) => {
                eprintln!("failed to get default-configs: {err}");
                process::exit(1)
            }
        };

        match config_file.write_all(default_configs.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to write default-configs: {err}");
            }
        }

        let mut athletes_file = match File::options().write(true).create(true).truncate(true).open(athletes_file_path) {
            Ok(athletes_file) => athletes_file,
            Err(err) => {
                eprintln!("failed to open athletes-file: {err}");
                process::exit(1)
            }
        };
        
        match athletes_file.write_all(b"[]") {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to write athletes: {err}");
                process::exit(1);
            }
        }

        match write_club(club_file_path, &Club::default()) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to write club-data: {err}");
                process::exit(1)
            }
        }
    }

    let lang_file = match get_config_dir() {
        Ok(lang_file) => lang_file,
        Err(err) => {
            eprintln!("failed to get config dir: {err}");
            process::exit(1)
        }
    }.join("e-melder").join("lang").join(format!("{}.json", get_config("lang").expect("unreachable").as_str().expect("unreachable")));

    if !lang_file.exists() {
        match create_dir_all(lang_file.parent().expect("unreachable")) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to create neccessary directories for lang-file: {err}");
                process::exit(1)
            }   
        }

        let mut lang_file = match File::options().write(true).create_new(true).open(lang_file) {
            Ok(lang_file) => lang_file,
            Err(err) => {
                eprintln!("failed to create lang-file: {err}");
                process::exit(1)
            }
        };

        let translations = match get_config("lang").expect("unreachable").as_str().expect("unreachable") {
            "de" => DEFAULT_TRANSLATIONS_DE,
            "en" => DEFAULT_TRANSLATIONS_EN,
            // other in the future supported languages would be listed here
            _ => "{}"
        };

        match lang_file.write_all(translations.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("failed to write default language: {err}");
                process::exit(1)
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
            eprintln!("failed to setup app: {err}");
            process::exit(1)
        }
    }.as_str(), options, Box::new(|cc| {
        Box::new(match EMelderApp::new(cc) {
            Ok(app) => app,
            Err(err) => {
                eprintln!("failed to setup app: {err}");
                process::exit(1)
            }
        }
    )
    }))
}
