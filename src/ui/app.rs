use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use cosmic::app::Task;
use cosmic::{ApplicationExt, Core, Element};
use cosmic::iced::keyboard::Key;
use cosmic::iced::Subscription;
use cosmic::iced::window::Id;
use cosmic::widget::calendar::CalendarModel;
use cosmic::widget::nav_bar;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::tournament_info::{Athlete,Club, GenderCategory,
    RegisteringAthlete};
use crate::utils::{DEFAULT_BIRTH_YEAR, LANG_NAMES, translate, GENDERS, BELTS, THEMES};
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

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    #[default]
    System
}

impl From<Theme> for cosmic::Theme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Dark => cosmic::Theme::dark(),
            Theme::Light => cosmic::Theme::light(),
            Theme::System => cosmic::theme::system_preference()
        }
    }
}

impl From<bool> for Theme {
    fn from(value: bool) -> Self {
        if value {
            Self::Dark
        }
        else {
            Self::Light
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    pub lang: String,
    pub theme: Theme,
    #[serde(rename = "athletes-file")]
    pub athletes_file: PathBuf,
    #[serde(rename = "club-file")]
    pub club_file: PathBuf,
    #[serde(rename = "tournament-basedir")]
    pub tournament_basedir: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    pub langs: Vec<String>,
    #[serde(default, serialize_with="crate::utils::serialize_gender",
    deserialize_with="crate::utils::deserialize_gender", rename = "default-gender")]
    pub default_gender: GenderCategory
}

#[derive(Deserialize)]
pub struct OldConfigs {
    pub lang: String,
    #[serde(rename = "dark-mode")]
    pub dark_mode: bool,
    #[serde(rename = "athletes-file")]
    pub athletes_file: PathBuf,
    #[serde(rename = "club-file")]
    pub club_file: PathBuf,
    #[serde(rename = "tournament-basedir")]
    pub tournament_basedir: PathBuf,
    #[serde(default, serialize_with="crate::utils::serialize_gender",
    deserialize_with="crate::utils::deserialize_gender", rename = "default-gender-category")]
    pub default_gender_category: GenderCategory
}

impl From<OldConfigs> for Configs {
    fn from(value: OldConfigs) -> Self {
        Self {
            lang: value.lang,
            theme: value.dark_mode.into(),
            athletes_file: value.athletes_file,
            club_file: value.club_file,
            tournament_basedir: value.tournament_basedir,
            langs: Vec::new(),
            default_gender: value.default_gender_category
        }
    }
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
    pub(super) calendar_model: CalendarModel,
    pub(super) legal_gender_categories: EnumMap<GenderCategory, Vec<String>>,
    pub(super) show_date: bool,
    pub(super) theme_selection: usize,
    pub(super) themes: Vec<String>,
    pub(super) athletes: Vec<Athlete>,
    pub(super) club: Club,
    pub(super) registering: Registering,
    pub(super) adding: Adding,
    pub(super) configs: Configs,
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
    Nop,
    Close
}

impl cosmic::Application for EMelderApp {
    type Message = Message;
    type Flags = (Configs, HashMap<String, String>, Club, Vec<Athlete>);
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
            *gender == configs.default_gender
        }).unwrap_or_default();
        let genders = GENDERS.iter().map(|gender| {
            translate!(&format!("register.table.gender_category.{}", gender.render()), &translations)
        }).collect();
        let belt_names = BELTS.iter().map(|belt| {
            translate!(&format!("add.belt.{}", belt.serialise()), &translations)
        }).collect();
        let calendar_model = CalendarModel::now();
        let legal_gender_categories = enum_map::enum_map! {
            GenderCategory::Female => vec![translate!("register.table.gender_category.w", &translations),
            translate!("register.table.gender_category.g", &translations)],
            GenderCategory::Male => vec![translate!("register.table.gender_category.m", &translations),
            translate!("register.table.gender_category.g", &translations)],
            GenderCategory::Mixed => vec![translate!("register.table.gender_category.w", &translations),
            translate!("register.table.gender_category.m", &translations),
            translate!("register.table.gender_category.g", &translations)]
        };
        let themes = vec![
            translate!("config.theme.system", &translations),
            translate!("config.theme.light", &translations),
            translate!("config.theme.dark", &translations)
        ];
        let theme_selection = THEMES.iter().position(|theme| {
            *theme == configs.theme
        }).unwrap_or_default();
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
            calendar_model,
            legal_gender_categories,
            show_date: false,
            theme_selection,
            themes,
            athletes,
            club,
            registering: Registering::default(),
            adding: Adding::default(),
            configs,
            translations,
            update_check_text: None
        };
        app.set_header_title(translate!("application.title", &app.translations));
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
            Message::Nop => Task::none(),
            Message::Close => cosmic::iced::exit()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::keyboard::on_key_press(|key, modifiers| {
            // checks for ctrl on most platforms
            if modifiers.command() && key.as_ref() == Key::Character("q") {
                Some(Message::Close)
            }
            else {
                None
            }
        })
    }
}
