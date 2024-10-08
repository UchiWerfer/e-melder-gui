use std::io;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use eframe::CreationContext;
use egui::{TextWrapMode, Ui, Visuals};
use egui_extras::{Column, TableBuilder};

use crate::tournament_info::{Athlete, Belt, Club, GenderCategory,
    RegisteringAthlete, WeightCategory};
use crate::utils::{check_update_available, crash, get_config, get_config_dir, read_athletes, read_club,
    write_athletes, write_club, write_config, UpdateAvailability, CODE_LINK, LANG_NAMES,
    LICENSE, LICENSE_LINK, VERSION, translate};
use super::registering::show_registering;

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
pub(super) struct Registering {
    pub(super) athletes: Vec<RegisteringAthlete>,
    pub(super) name: String,
    pub(super) place: String,
    pub(super) date: NaiveDate,
    pub(super) search: String
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
pub(super) struct Config {
    lang: String,
    dark_mode: bool,
    athletes_file: PathBuf,
    club_file: PathBuf,
    tournament_basedir: String,
    langs: Vec<String>,
    pub(super) default_gender_category: GenderCategory
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct EMelderApp {
    pub(super) athletes: Vec<Athlete>,
    pub(super) club: Club,
    pub(super) registering: Registering,
    adding: Adding,
    mode: Mode,
    pub(super) config: Config,
    update_check_text: Option<String>,
    popup_open: bool
}

impl EMelderApp {
    pub fn new(cc: &CreationContext) -> io::Result<Self> {
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
                    },
                },
                default_gender_category: match get_config("default-gender-category") {
                    Ok(dgc_value) => match dgc_value.as_str() {
                        Some(dgc_str) => {
                            match GenderCategory::from_str(dgc_str) {
                                Some(dgc) => dgc,
                                None => {
                                    log::warn!("default-gender-category-config does not represent a gender-category");
                                    GenderCategory::Mixed
                                }
                            }
                        }
                        None => {
                            log::warn!("default-gender-category-config is not a string");
                            GenderCategory::Mixed
                        }
                    },
                    Err(err) => {
                        log::warn!("failed to get default-gender-category-config, due to {err}");
                        GenderCategory::Mixed
                    }
                },
                langs: languages
            }, popup_open: false, update_check_text: None
        })
    }

    fn show_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(translate!("add.given_name"));
            ui.text_edit_singleline(&mut self.adding.given_name);
        });
        ui.horizontal(|ui| {
            ui.label(translate!("add.sur_name"));
            ui.text_edit_singleline(&mut self.adding.sur_name);
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(translate!("add.belt"))
            .selected_text(translate!(&format!("add.belt.{}", self.adding.belt.serialise())))
            .show_ui(ui, |ui| {
                for belt in [Belt::Kyu9, Belt::Kyu8, Belt::Kyu7, Belt::Kyu6, Belt::Kyu5, Belt::Kyu4, Belt::Kyu3, Belt::Kyu2, Belt::Kyu1,
                Belt::Dan1, Belt::Dan2, Belt::Dan3, Belt::Dan4, Belt::Dan5, Belt::Dan6, Belt::Dan7, Belt::Dan8, Belt::Dan9, Belt::Dan10] {
                    ui.selectable_value(&mut self.adding.belt, belt, translate!(
                        &format!("add.belt.{}", belt.serialise())));
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(translate!("add.year"));
            ui.add(egui::Slider::new(&mut self.adding.year, 1900..=2100));
        });

        if ui.button(translate!("add.commit")).clicked() {
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
            if ui.button(translate!("graduate.empty")).clicked() {
                self.mode = Mode::Adding;
            }
            return;
        }


        let mut to_graduate = None;
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 4).column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("graduate.given_name"));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.sur_name"));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.year"));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.belt"));
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
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise())));
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("graduate.graduate")).clicked() {
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
            ui.label(translate!("edit.club_name"));
            ui.text_edit_singleline(self.club.get_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.given_name"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_given_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.sur_name"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_sur_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.address"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_address_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.postal_code"));
            ui.add(egui::DragValue::new(self.club.get_sender_mut().get_postal_code_mut()));
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.town"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_town_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.private"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_private_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.public"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_public_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.fax"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_fax_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.mobile"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_mobile_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.mail"));
            ui.text_edit_singleline(self.club.get_sender_mut().get_mail_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.club_number"));
            ui.add(egui::DragValue::new(self.club.get_number_mut()));
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.county"));
            ui.text_edit_singleline(self.club.get_county_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.region"));
            ui.text_edit_singleline(self.club.get_region_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.state"));
            ui.text_edit_singleline(self.club.get_state_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.group"));
            ui.text_edit_singleline(self.club.get_group_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.nation"));
            ui.text_edit_singleline(self.club.get_nation_mut());
        });

        if ui.button(translate!("edit.save")).clicked() {
            let path_value = match get_config("club-file") {
                Ok(path) => path,
                Err(err) => {
                    log::error!("failed to get config club-file, due to {err}");
                    crash();
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
            ui.label(translate!("delete.empty"));
            return;
        }

        let mut to_delete = None;
        let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("delete.given_name"));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.sur_name"));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.year"));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.belt"));
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
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise())));
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("delete.delete")).clicked() {
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
            egui::ComboBox::from_label(translate!("config.lang"))
            .selected_text(*LANG_NAMES.get(self.config.lang.as_str()).unwrap_or(&self.config.lang.as_str()))
            .show_ui(ui, |ui| {
                for lang in &self.config.langs {
                    ui.selectable_value(&mut self.config.lang, lang.clone(),
                    *LANG_NAMES.get(lang.as_str()).unwrap_or(&lang.as_str()));
                }
            });
        });
        
        ui.checkbox(&mut self.config.dark_mode, translate!("config.dark_mode"));

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_athletes_file"));
            if ui.button(self.config.athletes_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(translate!("config.athletes_file.file_picker")).save_file() {
                        Some(athletes_file) => {
                            self.config.athletes_file = athletes_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_club_file"));
            if ui.button(self.config.club_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(translate!("config.club_file.file_picker")).save_file() {
                        Some(club_file) => {
                            self.config.club_file = club_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_tournament_basedir"));
            if ui.button(&self.config.tournament_basedir).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_directory(&self.config.tournament_basedir)
                    .set_can_create_directories(true).set_title(translate!("config.tournament_basedir.file_picker"))
                    .pick_folder() {
                        Some(directory) => {
                            self.config.tournament_basedir = directory.display().to_string();
                        },
                        None => {}
                    }
            }
        });

        egui::ComboBox::from_label(translate!("config.default_gender_category"))
        .selected_text(translate!(&format!("register.table.gender_category.{}", self.config.default_gender_category.render())))
        .show_ui(ui, |ui| {
            for gender_category in [GenderCategory::Mixed, GenderCategory::Female, GenderCategory::Male] {
                ui.selectable_value(&mut self.config.default_gender_category, gender_category,
                    translate!(&format!("register.table.gender_category.{}", gender_category.render())));
            }
        });

        if ui.button(translate!("config.save")).clicked() {
            match write_config("lang", self.config.lang.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }
            
            match write_config("dark-mode", self.config.dark_mode.into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }

            match write_config("athletes-file", self.config.athletes_file.display().to_string().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }

            match write_config("club-file", self.config.club_file.display().to_string().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }

            match write_config("tournament-basedir", self.config.tournament_basedir.clone().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }

            match write_config("default-gender-category", self.config.default_gender_category.render().into()) {
                Ok(()) => {},
                Err(err) => {
                    log::warn!("failed to set config, due to {err}");
                }
            }
        }
    }

    fn show_about(&mut self, ui: &mut Ui) {
        ui.label(translate!("about.about"));
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(translate!("about.version"));
            ui.label(VERSION);
        });

        ui.horizontal(|ui| {
            ui.label(translate!("about.license"));
            if ui.link(LICENSE).on_hover_text(LICENSE_LINK).clicked() {
                let _ = open::that_detached(LICENSE_LINK);
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("about.source_code"));
            if ui.link(CODE_LINK).on_hover_text(CODE_LINK).clicked() {
                let _ = open::that_detached(CODE_LINK);
            }
        });

        if ui.button(translate!("about.check_update")).clicked() {
            let update_available = check_update_available(VERSION);
            self.popup_open = true;
            if let Ok(update_available) = update_available {
                match update_available {
                    UpdateAvailability::UpdateAvailable => {
                        self.update_check_text = Some(translate!("about.update_available"));
                    }
                    UpdateAvailability::NoUpdateAvailable => {
                        self.update_check_text = Some(translate!("about.no_update_available"));
                    }
                    UpdateAvailability::RunningUnstable => {
                        self.update_check_text = Some(translate!("about.running_unstable"));
                    }
                }
            }
            else {
                log::warn!("failed to get new version information from network: {}", update_available.unwrap_err()); // cannot panic, as it was checked above for `Ok`
                self.update_check_text = Some(translate!("about.no_network"));
            }
        }
    }
}

impl eframe::App for EMelderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.popup_open && self.update_check_text.is_some() {
            self.update_check_text = None;
        }

        if let Some(update_check_text) = &self.update_check_text {
            egui::Window::new(translate!("about.update_popup_title"))
            .collapsible(false).resizable(false).open(&mut self.popup_open).show(ctx, |ui| {
                ui.label(update_check_text);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.update_check_text.is_some() {
                ui.disable();
            }
            egui::menu::bar(ui, |ui| {
                if ui.button(translate!("application.register")).clicked() {
                    self.mode = Mode::Registering;
                }

                if ui.button(translate!("application.add")).clicked() {
                    self.mode = Mode::Adding;
                }

                if ui.button(translate!("application.graduate")).clicked() {
                    self.mode = Mode::Graduating;
                }

                if ui.button(translate!("application.delete")).clicked() {
                    self.mode = Mode::Deleting;
                }

                if ui.button(translate!("application.edit")).clicked() {
                    self.mode = Mode::EditClub;
                }

                if ui.button(translate!("application.config")).clicked() {
                    self.mode = Mode::Config;
                }

                if ui.button(translate!("application.about")).clicked() {
                    self.mode = Mode::About;
                }
            });

            match self.mode {
                Mode::Registering => show_registering(self, ui),
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
