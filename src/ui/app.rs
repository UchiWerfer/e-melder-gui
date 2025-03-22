use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use cosmic::app::Task;
use cosmic::{ApplicationExt, Core, Element};
use cosmic::iced::window::Id;
use cosmic::widget::nav_bar;
use serde::{Deserialize, Serialize};

use crate::tournament_info::{Athlete, Belt, Club, GenderCategory,
    RegisteringAthlete, WeightCategory};
use crate::utils::{check_update_available, crash, get_configs, get_config_dir, read_athletes,
                   read_club, write_athletes, write_club, write_configs, get_translations,
                   UpdateAvailability, CODE_LINK, DEFAULT_BIRTH_YEAR, LANG_NAMES, LICENSE_LINK,
                   LOWER_BOUND_BIRTH_YEAR, UPPER_BOUND_BIRTH_YEAR, VERSION, translate};

#[derive(Default, Debug)]
enum Page {
    #[default]
    Registering,
    Adding,
    Deleting,
    EditAthlete,
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
    year: u16,
    gender: GenderCategory
}

impl Adding {
    fn clear(&mut self, config: &Config) {
        *self = Self::from_config(config);
    }

    fn from_config(config: &Config) -> Self {
        Self {
            given_name: String::default(),
            sur_name: String::default(),
            belt: Belt::default(),
            year: DEFAULT_BIRTH_YEAR,
            gender: config.default_gender_category
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub lang: String,
    #[serde(rename = "dark-mode")]
    // TODO: use theme-enum with: Dark, White, System variants
    pub dark_mode: bool,
    #[serde(rename = "athletes-file")]
    pub athletes_file: PathBuf,
    #[serde(rename = "club-file")]
    pub club_file: PathBuf,
    #[serde(rename = "tournament-basedir")]
    pub tournament_basedir: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    pub langs: Vec<String>,
    #[serde(default, serialize_with="crate::utils::serialize_gender_category",
    deserialize_with="crate::utils::deserialize_gender_category", rename = "default-gender-category")]
    pub default_gender_category: GenderCategory
}

#[allow(clippy::module_name_repetitions)]
pub struct EMelderApp {
    core: Core,
    nav: nav_bar::Model,
    athletes: Vec<Athlete>,
    club: Club,
    registering: Registering,
    adding: Adding,
    configs: Config,
    pub(super) update_check_text: Option<String>,
    //popup_open: bool,
    pub(super) translations: HashMap<String, String>
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CheckUpdate,
    License,
    Code,
    CheckUpdateClose
}

impl cosmic::Application for EMelderApp {
    type Message = Message;
    type Flags = (Config, HashMap<String, String>, Club, Vec<Athlete>);
    type Executor = cosmic::executor::Default;

    const APP_ID: &'static str = "io.github.UchiWerfer.e-melder-gui";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        Task::none()
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let (configs, translations, club, athletes) = flags;
        let mut nav = nav_bar::Model::default();
        nav.insert()
            .text(translate!("application.register", &translations))
            .data(Page::Registering)
            .activate();
        nav.insert()
            .text(translate!("application.add", &translations))
            .data(Page::Adding);
        nav.insert()
            .text(translate!("application.edit_athlete", &translations))
            .data(Page::EditAthlete);
        nav.insert()
            .text(translate!("application.edit", &translations))
            .data(Page::EditClub);
        nav.insert()
            .text(translate!("application.delete", &translations))
            .data(Page::Deleting);
        nav.insert()
            .text(translate!("application.config", &translations))
            .data(Page::Config);
        nav.insert()
            .text(translate!("application.about", &translations))
            .data(Page::About);
        let mut app = Self {
            core,
            nav,
            athletes,
            club,
            registering: Registering::default(),
            adding: Adding::from_config(&configs),
            configs,
            translations,
            update_check_text: None
        };
        let command = app.set_window_title(translate!("application.title", &app.translations), Id::unique());
        (app, command)
    }

    fn view(&self) -> Element<Self::Message> {
        if let Some(page) = self.nav.active_data::<Page>() {
            match page {
                Page::Registering => self.view_registering(),
                Page::Adding => self.view_adding(),
                Page::EditAthlete => self.view_edit_athlete(),
                Page::Deleting => self.view_deleting(),
                Page::EditClub => self.view_edit_club(),
                Page::Config => self.view_config(),
                Page::About => self.view_about()
            }
        }
        else {
            cosmic::widget::text(translate!("application.empty", &self.translations)).into()
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Code => {
                let _ = open::that_detached(CODE_LINK);
            }
            Message::License => {
                let _ = open::that_detached(LICENSE_LINK);
            }
            Message::CheckUpdate => {
                let update_available = check_update_available(VERSION);
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
                    log::warn!("failed to get new version information from network: {}", update_available.unwrap_err());  // cannot panic as it was checked above for `Ok`
                    self.update_check_text = Some(translate!("about.no_network", &self.translations));
                }
            }
            Message::CheckUpdateClose => {
                self.update_check_text = None;
            }
        }
        Task::none()
    }
}
