use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use eframe::CreationContext;
use egui::{TextWrapMode, Ui, Visuals};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

use crate::tournament_info::{Athlete, Belt, Club, GenderCategory,
    RegisteringAthlete, WeightCategory};
use crate::utils::{check_update_available, crash, get_configs, get_config_dir,
    read_athletes, read_club, write_athletes, write_club, write_configs,
    get_translations, UpdateAvailability, CODE_LINK, DEFAULT_BIRTH_YEAR, LANG_NAMES,
    LICENSE, LICENSE_LINK, LOWER_BOUND_BIRTH_YEAR, UPPER_BOUND_BIRTH_YEAR, VERSION, translate};
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
            year: DEFAULT_BIRTH_YEAR
        }
    }
}

impl Adding {
    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub lang: String,
    #[serde(rename = "dark-mode")]
    pub dark_mode: bool,
    #[serde(rename = "athletes-file")]
    pub athletes_file: PathBuf,
    #[serde(rename = "club-file")]
    pub club_file: PathBuf,
    #[serde(rename = "tournament-basedir")]
    pub tournament_basedir: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    pub langs: Vec<String>,
    #[serde(default, serialize_with="serialize_gender_category", deserialize_with="deserialize_gender_category", rename = "default-gender-category")]
    pub default_gender_category: GenderCategory
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_gender_category<S>(gender_category: &GenderCategory, serializer: S) -> Result<S::Ok, S::Error>
where S: serde::Serializer {
    serializer.serialize_str(gender_category.render())
}

fn deserialize_gender_category<'de, D>(deserializer: D) -> Result<GenderCategory, D::Error>
where D: serde::Deserializer<'de> {
    GenderCategory::from_str(&String::deserialize(deserializer)?).ok_or(serde::de::Error::custom("Invalid Gender category"))
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
    popup_open: bool,
    pub(super) translations: HashMap<String, String>
}

impl EMelderApp {
    pub fn new(cc: &CreationContext) -> io::Result<Self> {
        let mut configs = get_configs()?;
        let athletes = match read_athletes(&configs.athletes_file) {
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
        let club = match read_club(&configs.club_file) {
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
        configs.langs = languages;

        let visuals = if configs.dark_mode { Visuals::dark() } else { Visuals::light() };
        
        cc.egui_ctx.set_visuals(visuals);
        let lang_clone = configs.lang.clone();
        Ok(Self {
            athletes, club, registering: Registering::default(), adding: Adding::default(), mode: Mode::default(),
            config: configs, popup_open: false, update_check_text: None,
            translations: get_translations(&lang_clone)?
        })
    }

    fn show_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(translate!("add.given_name", &self.translations));
            ui.text_edit_singleline(&mut self.adding.given_name);
        });
        ui.horizontal(|ui| {
            ui.label(translate!("add.sur_name", &self.translations));
            ui.text_edit_singleline(&mut self.adding.sur_name);
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(translate!("add.belt", &self.translations))
            .selected_text(translate!(&format!("add.belt.{}", self.adding.belt.serialise()), &self.translations))
            .show_ui(ui, |ui| {
                for belt in [Belt::Kyu9, Belt::Kyu8, Belt::Kyu7, Belt::Kyu6, Belt::Kyu5, Belt::Kyu4, Belt::Kyu3, Belt::Kyu2, Belt::Kyu1,
                Belt::Dan1, Belt::Dan2, Belt::Dan3, Belt::Dan4, Belt::Dan5, Belt::Dan6, Belt::Dan7, Belt::Dan8, Belt::Dan9, Belt::Dan10] {
                    ui.selectable_value(&mut self.adding.belt, belt,
                        translate!(&format!("add.belt.{}", belt.serialise()), &self.translations));
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(translate!("add.year", &self.translations));
            ui.add(egui::Slider::new(&mut self.adding.year, LOWER_BOUND_BIRTH_YEAR..=UPPER_BOUND_BIRTH_YEAR));
        });

        if ui.button(translate!("add.commit", &self.translations)).clicked() {
            self.athletes.push(Athlete::new(
                self.adding.given_name.clone(), self.adding.sur_name.clone(),
                self.adding.year, self.adding.belt, WeightCategory::default()
            ));
            self.adding.clear();
            match write_athletes(&self.config.athletes_file, &self.athletes) {
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
            if ui.button(translate!("graduate.empty", &self.translations)).clicked() {
                self.mode = Mode::Adding;
            }
            return;
        }


        let mut to_graduate = None;
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 4).column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("graduate.given_name", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.sur_name", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.year", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("graduate.belt", &self.translations));
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
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()), &self.translations));
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("graduate.graduate", &self.translations)).clicked() {
                            to_graduate = Some(index);
                        }
                    });
                });
            }
        });

        if let Some(index) = to_graduate {
            let belt = self.athletes[index].get_belt();
            *self.athletes[index].get_belt_mut() = belt.inc();
            #[allow(clippy::single_match_else)]
            match write_athletes(&self.config.athletes_file, &self.athletes) {
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
            ui.label(translate!("edit.club_name", &self.translations));
            ui.text_edit_singleline(self.club.get_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.given_name", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_given_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.sur_name", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_sur_name_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.address", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_address_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.postal_code", &self.translations));
            ui.add(egui::DragValue::new(self.club.get_sender_mut().get_postal_code_mut())
                .range(11000..=99999));
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.town", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_town_mut());
        });
        
        ui.horizontal(|ui| {
            ui.label(translate!("edit.private", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_private_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.public", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_public_phone_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.fax", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_fax_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.mobile", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_mobile_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.mail", &self.translations));
            ui.text_edit_singleline(self.club.get_sender_mut().get_mail_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.club_number", &self.translations));
            ui.add(egui::DragValue::new(self.club.get_number_mut())
                .range(0..=9_999_999)
                .custom_formatter(|n, _| {
                    format!("{n:07}")
                }));
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.county", &self.translations));
            ui.text_edit_singleline(self.club.get_county_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.region", &self.translations));
            ui.text_edit_singleline(self.club.get_region_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.state", &self.translations));
            ui.text_edit_singleline(self.club.get_state_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.group", &self.translations));
            ui.text_edit_singleline(self.club.get_group_mut());
        });

        ui.horizontal(|ui| {
            ui.label(translate!("edit.nation", &self.translations));
            ui.text_edit_singleline(self.club.get_nation_mut());
        });

        if ui.button(translate!("edit.save", &self.translations)).clicked() {
            match write_club(&self.config.club_file, &self.club) {
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
            ui.label(translate!("delete.empty", &self.translations));
            return;
        }

        let mut to_delete = None;
        let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("delete.given_name", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.sur_name", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.year", &self.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("delete.belt", &self.translations));
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
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()), &self.translations));
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("delete.delete", &self.translations)).clicked() {
                            to_delete = Some(index);
                        }
                    });
                });
            }
        });

        if let Some(index) = to_delete {
            self.athletes.remove(index);
            match write_athletes(&self.config.athletes_file, &self.athletes) {
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
            egui::ComboBox::from_label(translate!("config.lang", &self.translations))
            .selected_text(*LANG_NAMES.get(self.config.lang.as_str()).unwrap_or(&self.config.lang.as_str()))
            .show_ui(ui, |ui| {
                for lang in &self.config.langs {
                    ui.selectable_value(&mut self.config.lang, lang.clone(),
                    *LANG_NAMES.get(lang.as_str()).unwrap_or(&lang.as_str()));
                }
            });
        });
        
        ui.checkbox(&mut self.config.dark_mode, translate!("config.dark_mode", &self.translations));

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_athletes_file", &self.translations));
            if ui.button(self.config.athletes_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(translate!("config.athletes_file.file_picker", &self.translations)).save_file() {
                        Some(athletes_file) => {
                            self.config.athletes_file = athletes_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_club_file", &self.translations));
            if ui.button(self.config.club_file.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_can_create_directories(true)
                    .set_title(translate!("config.club_file.file_picker", &self.translations)).save_file() {
                        Some(club_file) => {
                            self.config.club_file = club_file;
                        }
                        None => {}
                    }
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("config.select_tournament_basedir", &self.translations));
            if ui.button(self.config.tournament_basedir.display().to_string()).clicked() {
                #[allow(clippy::single_match)]
                match rfd::FileDialog::new().set_directory(&self.config.tournament_basedir)
                    .set_can_create_directories(true).set_title(translate!("config.tournament_basedir.file_picker",
                    &self.translations))
                    .pick_folder() {
                        Some(directory) => {
                            self.config.tournament_basedir = directory;
                        },
                        None => {}
                    }
            }
        });

        egui::ComboBox::from_label(translate!("config.default_gender_category", &self.translations))
        .selected_text(translate!(&format!("register.table.gender_category.{}", self.config.default_gender_category.render()),
        &self.translations))
        .show_ui(ui, |ui| {
            for gender_category in [GenderCategory::Mixed, GenderCategory::Female, GenderCategory::Male] {
                ui.selectable_value(&mut self.config.default_gender_category, gender_category,
                    translate!(&format!("register.table.gender_category.{}", gender_category.render()), &self.translations));
            }
        });

        if ui.button(translate!("config.save", &self.translations)).clicked() {
            match write_configs(&self.config) {
                Ok(()) => {
                    self.translations.clear();
                    self.translations = match get_translations(&self.config.lang) {
                        Ok(translations) => translations,
                        Err(err) => {
                            log::warn!("failed to obtain translations, due to {err}");
                            HashMap::new()
                        }
                    }
                },
                Err(err) => {
                    log::warn!("failed to write configs, due to {err}");
                }
            }
        }
    }

    fn show_about(&mut self, ui: &mut Ui) {
        ui.label(translate!("about.about", &self.translations));
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(translate!("about.version", &self.translations));
            ui.label(VERSION);
        });

        ui.horizontal(|ui| {
            ui.label(translate!("about.license", &self.translations));
            if ui.link(LICENSE).on_hover_text(LICENSE_LINK).clicked() {
                let _ = open::that_detached(LICENSE_LINK);
            }
        });

        ui.horizontal(|ui| {
            ui.label(translate!("about.source_code", &self.translations));
            if ui.link(CODE_LINK).on_hover_text(CODE_LINK).clicked() {
                let _ = open::that_detached(CODE_LINK);
            }
        });

        if ui.button(translate!("about.check_update", &self.translations)).clicked() {
            let update_available = check_update_available(VERSION);
            self.popup_open = true;
            if let Ok(update_available) = update_available {
                match update_available {
                    UpdateAvailability::UpdateAvailable => {
                        self.update_check_text = Some(translate!("about.update_available", &self.translations));
                    }
                    UpdateAvailability::NoUpdateAvailable => {
                        self.update_check_text = Some(translate!("about.no_update_available", &self.translations));
                    }
                    UpdateAvailability::RunningUnstable => {
                        self.update_check_text = Some(translate!("about.running_unstable", &self.translations));
                    }
                }
            }
            else {
                log::warn!("failed to get new version information from network: {}", update_available.unwrap_err()); // cannot panic, as it was checked above for `Ok`
                self.update_check_text = Some(translate!("about.no_network", &self.translations));
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
            egui::Window::new(translate!("about.update_popup_title", &self.translations))
            .collapsible(false).resizable(false).open(&mut self.popup_open).show(ctx, |ui| {
                ui.label(update_check_text);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.update_check_text.is_some() {
                ui.disable();
            }
            egui::menu::bar(ui, |ui| {
                if ui.button(translate!("application.register", &self.translations)).clicked() {
                    self.mode = Mode::Registering;
                }

                if ui.button(translate!("application.add", &self.translations)).clicked() {
                    self.mode = Mode::Adding;
                }

                if ui.button(translate!("application.graduate", &self.translations)).clicked() {
                    self.mode = Mode::Graduating;
                }

                if ui.button(translate!("application.delete", &self.translations)).clicked() {
                    self.mode = Mode::Deleting;
                }

                if ui.button(translate!("application.edit", &self.translations)).clicked() {
                    self.mode = Mode::EditClub;
                }

                if ui.button(translate!("application.config", &self.translations)).clicked() {
                    self.mode = Mode::Config;
                }

                if ui.button(translate!("application.about", &self.translations)).clicked() {
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
