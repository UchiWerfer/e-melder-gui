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
use crate::utils::{crash, get_configs, get_config_dir, read_athletes, read_club, write_athletes,
                   write_club, write_configs, get_translations, CODE_LINK, DEFAULT_BIRTH_YEAR,
                   LANG_NAMES, LICENSE_LINK, VERSION, translate, GENDERS, BELTS};
use crate::ui::administrative::{EditClubMessage, ConfigMessage, AboutMessage};
use crate::ui::usage::{RegisteringMessage, AddingMessage, EditAthleteMessage, DeletingMessage};

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
pub(super) struct Adding {
    pub(super) given_name: String,
    pub(super) sur_name: String,
    pub(super) year: u16,
}

impl Default for Adding {
    fn default() -> Self {
        Self {
            given_name: String::new(),
            sur_name: String::new(),
            year: DEFAULT_BIRTH_YEAR
        }
    }
}

impl Adding {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lang: String,
    #[serde(rename = "dark-mode")]
    // TODO: use `Theme`-enum with: `Dark`, `White`, `System` variants
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
    // TODO: rename to `default_gender`
    pub default_gender_category: GenderCategory
}

#[allow(clippy::module_name_repetitions)]
pub struct EMelderApp {
    core: Core,
    nav: nav_bar::Model,
    pub(super) update_check_text: Option<String>,
    pub(super) config_lang_selection: usize,
    pub(super) lang_names: Vec<String>,
    pub(super) genders: Vec<String>,
    pub(super) gender_selection: usize,
    pub(super) belt_names: Vec<String>,
    pub(super) belt_selection: usize,
    pub(super) adding_gender_selection: usize,
    pub(super) athletes: Vec<Athlete>,
    pub(super) club: Club,
    registering: Registering,
    pub(super) adding: Adding,
    pub(super) configs: Config,
    pub(super) translations: HashMap<String, String>
}

#[derive(Clone, Debug)]
pub enum Message {
    About(AboutMessage),
    Config(ConfigMessage),
    EditClub(EditClubMessage),
    Deleting(DeletingMessage),
    EditAthlete(EditAthleteMessage),
    Adding(AddingMessage),
    Registering(RegisteringMessage),
    Nop
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
            .text(translate!("application.delete", &translations))
            .data(Page::Deleting);
        nav.insert()
            .text(translate!("application.edit", &translations))
            .data(Page::EditClub);
        nav.insert()
            .text(translate!("application.config", &translations))
            .data(Page::Config);
        nav.insert()
            .text(translate!("application.about", &translations))
            .data(Page::About);
        let config_lang_selection = configs.langs.iter().position(|lang_code| {
            lang_code == &configs.lang
        }).unwrap_or_default();
       let lang_names = configs.langs.iter().map(|lang_code| {
            LANG_NAMES.get(lang_code.as_str()).unwrap_or(&lang_code.as_str()).to_owned().to_owned()
        }).collect();
        let gender_selection = GENDERS.iter().position(|gender| {
            *gender == configs.default_gender_category
        }).unwrap_or_default();
        let genders = GENDERS.iter().map(|gender| {
            translate!(&format!("register.table.gender_category.{}", gender.render()), &translations)
        }).collect();
        let belt_names = BELTS.iter().map(|belt| {
            translate!(&format!("add.belt.{}", belt.serialise()), &translations)
        }).collect();
        let mut app = Self {
            core,
            nav,
            config_lang_selection,
            lang_names,
            genders,
            gender_selection,
            belt_names,
            belt_selection: 0,
            adding_gender_selection: gender_selection,
            athletes,
            club,
            registering: Registering::default(),
            adding: Adding::default(),
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
            Message::Registering(registering) => self.update_registering(registering),
            Message::Adding(adding) => self.update_adding(adding),
            Message::EditAthlete(edit_athlete) => self.update_edit_athlete(edit_athlete),
            Message::Deleting(deleting) => self.update_deleting(deleting),
            Message::EditClub(edit_club) => self.update_edit_club(edit_club),
            Message::Config(config) => self.update_config(config),
            Message::About(about) => self.update_about(about),
            Message::Nop => Task::none()
        }
    }
}
