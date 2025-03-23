use std::path::PathBuf;
use cosmic::{theme, widget, Application, Element};
use cosmic::app::Task;
use cosmic::widget::tooltip::Position;

use crate::translate;
use crate::ui::app::Message;
use crate::ui::EMelderApp;
use crate::utils::{check_update_available, write_club, write_configs, UpdateAvailability, CODE_LINK, GENDER_CATEGORIES, LANG_NAMES, LEGAL_GENDER_CATEGORIES, LICENSE, LICENSE_LINK, VERSION};

#[derive(Clone, Debug)]
pub enum EditClubMessage {
    ClubName(String),
    GivenName(String),
    SurName(String),
    Address(String),
    PostalCode(String),
    Town(String),
    PrivatePhone(String),
    PublicPhone(String),
    Fax(String),
    Mobile(String),
    Mail(String),
    ClubNumber(String),
    County(String),
    Region(String),
    State(String),
    Group(String),
    Nation(String),
    Save
}

#[derive(Clone, Debug)]
pub enum ConfigMessage {
    LanguageSelected(usize),
    DarkMode(bool),
    SelectAthletesFile,
    SelectClubFile,
    SelectTournamentBasedir,
    GenderSelected(usize),
    SaveConfig,
    SelectedAthletesFile(Option<PathBuf>),
    SelectedClubFile(Option<PathBuf>),
    SelectedTournamentBasedir(Option<PathBuf>)
}

#[derive(Clone, Debug)]
pub enum AboutMessage {
    CheckUpdate,
    License,
    Code,
    CheckUpdateClose,
    UpdateChecked(&'static str)
}

impl EMelderApp {
    pub fn view_edit_club(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(18)
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.club_name", &self.translations)))
                .push(widget::text_input("", self.club.get_name())
                    .on_input(|input| Message::EditClub(EditClubMessage::ClubName(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.given_name", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_given_name())
                    .on_input(|input| Message::EditClub(EditClubMessage::GivenName(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.sur_name", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_sur_name())
                    .on_input(|input| Message::EditClub(EditClubMessage::SurName(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.address", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_address())
                    .on_input(|input| Message::EditClub(EditClubMessage::Address(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.postal_code", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_postal_code().to_string())
                    .on_input(|input| Message::EditClub(EditClubMessage::PostalCode(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.town", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_town())
                    .on_input(|input| Message::EditClub(EditClubMessage::Town(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.private", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_private_phone())
                    .on_input(|input| Message::EditClub(EditClubMessage::PrivatePhone(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.public", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_public_phone())
                    .on_input(|input| Message::EditClub(EditClubMessage::PublicPhone(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.fax", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_fax())
                    .on_input(|input| Message::EditClub(EditClubMessage::Fax(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.mobile", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_mobile())
                    .on_input(|input| Message::EditClub(EditClubMessage::Mobile(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.mail", &self.translations)))
                .push(widget::text_input("", self.club.get_sender().get_mail())
                    .on_input(|input| Message::EditClub(EditClubMessage::Mobile(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.club_number", &self.translations)))
                .push(widget::text_input("", format!("{:0>7}", self.club.get_number().to_string()))
                    .on_input(|input| Message::EditClub(EditClubMessage::ClubNumber(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.county", &self.translations)))
                .push(widget::text_input("", self.club.get_county())
                    .on_input(|input| Message::EditClub(EditClubMessage::County(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.region", &self.translations)))
                .push(widget::text_input("", self.club.get_region())
                    .on_input(|input| Message::EditClub(EditClubMessage::Region(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.state", &self.translations)))
                .push(widget::text_input("", self.club.get_state())
                    .on_input(|input| Message::EditClub(EditClubMessage::State(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.group", &self.translations)))
                .push(widget::text_input("", self.club.get_group())
                    .on_input(|input| Message::EditClub(EditClubMessage::Group(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("edit.nation", &self.translations)))
                .push(widget::text_input("", self.club.get_nation())
                    .on_input(|input| Message::EditClub(EditClubMessage::Nation(input)))))
            .push(widget::button::text(translate!("edit.save", &self.translations))
                .on_press(Message::EditClub(EditClubMessage::Save)))
            .into()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn update_edit_club(&mut self, message: EditClubMessage) -> Task<<Self as Application>::Message> {
        match message {
            EditClubMessage::ClubName(club_name) => {
                self.club.get_name_mut().clone_from(&club_name);
            }
            EditClubMessage::GivenName(given_name) => {
                self.club.get_sender_mut().get_given_name_mut().clone_from(&given_name);
            }
            EditClubMessage::SurName(sur_name) => {
                self.club.get_sender_mut().get_sur_name_mut().clone_from(&sur_name);
            }
            EditClubMessage::Address(address) => {
                self.club.get_sender_mut().get_address_mut().clone_from(&address);
            }
            EditClubMessage::PostalCode(postal_code_str) => {
                let postal_code = postal_code_str.parse();
                if let Ok(postal_code) = postal_code {
                    if (11000..=99999).contains(&postal_code) {
                        *self.club.get_sender_mut().get_postal_code_mut() = postal_code;
                    }
                }
            }
            EditClubMessage::Town(town) => {
                self.club.get_sender_mut().get_town_mut().clone_from(&town);
            }
            EditClubMessage::PrivatePhone(private_phone) => {
                self.club.get_sender_mut().get_private_phone_mut().clone_from(&private_phone);
            }
            EditClubMessage::PublicPhone(public_phone) => {
                self.club.get_sender_mut().get_public_phone_mut().clone_from(&public_phone);
            }
            EditClubMessage::Fax(fax) => {
                self.club.get_sender_mut().get_fax_mut().clone_from(&fax);
            }
            EditClubMessage::Mobile(mobile) => {
                self.club.get_sender_mut().get_mobile_mut().clone_from(&mobile);
            }
            EditClubMessage::Mail(mail) => {
                self.club.get_sender_mut().get_mail_mut().clone_from(&mail);
            }
            EditClubMessage::ClubNumber(club_number_str) => {
                let club_number = club_number_str.parse();
                if let Ok(club_number) = club_number {
                    if (0..=9_999_999).contains(&club_number) {
                        *self.club.get_number_mut() = club_number;
                    }
                }
            }
            EditClubMessage::County(county) => {
                self.club.get_county_mut().clone_from(&county);
            }
            EditClubMessage::Region(region) => {
                self.club.get_region_mut().clone_from(&region);
            }
            EditClubMessage::State(state) => {
                self.club.get_state_mut().clone_from(&state);
            }
            EditClubMessage::Group(group) => {
                self.club.get_group_mut().clone_from(&group);
            }
            EditClubMessage::Nation(nation) => {
                self.club.get_nation_mut().clone_from(&nation);
            }
            EditClubMessage::Save => {
                let club_file = self.configs.club_file.clone();
                let club = self.club.clone();
                return cosmic::task::future(async move {
                    if let Err(err) = write_club(&club_file, &club) {
                        log::warn!("failed to write club, due to {err}");
                    }
                    Message::Nop
                });
            }
        }
        Task::none()
    }

    pub fn view_config(&self) -> Element<<Self as Application>::Message> {
            widget::column::with_capacity(7)
                .push(widget::row::with_capacity(2)
                    .push(widget::dropdown(&self.lang_names,
                    Some(self.config_lang_selection),
                    |selection| Message::Config(ConfigMessage::LanguageSelected(selection))))
                    .push(widget::text(translate!("config.lang", &self.translations))))
                .push(widget::row::with_capacity(2)
                    .push(widget::text(translate!("config.dark_mode", &self.translations)))
                    .push(widget::toggler(self.configs.dark_mode)
                        .on_toggle(|dark_mode| Message::Config(ConfigMessage::DarkMode(dark_mode)))))
                .push(widget::row::with_capacity(2)
                    .push(widget::text(translate!("config.select_athletes_file", &self.translations)))
                    .push(widget::button::text(self.configs.athletes_file.display().to_string())
                        .on_press(Message::Config(ConfigMessage::SelectAthletesFile))))
                .push(widget::row::with_capacity(2)
                    .push(widget::text(translate!("config.select_club_file", &self.translations)))
                    .push(widget::button::text(self.configs.club_file.display().to_string())
                        .on_press(Message::Config(ConfigMessage::SelectClubFile))))
                .push(widget::row::with_capacity(2)
                    .push(widget::text(translate!("config.select_tournament_basedir", &self.translations)))
                    .push(widget::button::text(self.configs.tournament_basedir.display().to_string())
                        .on_press(Message::Config(ConfigMessage::SelectTournamentBasedir))))
                .push(widget::row::with_capacity(2)
                    .push(widget::dropdown(&self.genders,
                                           Some(self.gender_selection),
                                           |selection| Message::Config(ConfigMessage::GenderSelected(selection))))
                    .push(widget::text(translate!("config.default_gender", &self.translations))))
                .push(widget::button::text(translate!("config.save", &self.translations))
                    .on_press(Message::Config(ConfigMessage::SaveConfig)))
                .into()
    }

    pub fn update_config(&mut self, message: ConfigMessage) -> Task<<Self as Application>::Message> {
        match message {
            ConfigMessage::LanguageSelected(selection) => {
                self.config_lang_selection = selection;
                self.configs.lang = self.configs.langs[selection].clone();
            }
            ConfigMessage::DarkMode(dark_mode) => {
                self.configs.dark_mode = dark_mode;
            }
            ConfigMessage::SelectAthletesFile => {
                let translation = translate!("config.athletes_file.file_picker", &self.translations);
                return cosmic::task::future(async {
                    Message::Config(ConfigMessage::SelectedAthletesFile(rfd::FileDialog::new().set_can_create_directories(true)
                        .set_title(translation).save_file()))
                });
            }
            ConfigMessage::SelectClubFile => {
                let translation = translate!("config.club_file.file_picker", &self.translations);
                return cosmic::task::future(async {
                    Message::Config(ConfigMessage::SelectedClubFile(rfd::FileDialog::new().set_can_create_directories(true)
                        .set_title(translation).save_file()))
                })
            }
            ConfigMessage::SelectTournamentBasedir => {
                let translation = translate!("config.tournament_basedir.file_picker", &self.translations);
                return cosmic::task::future(async move {
                    Message::Config(ConfigMessage::SelectedTournamentBasedir(rfd::FileDialog::new().set_can_create_directories(true)
                        .set_title(translation).pick_file()))
                })
            }
            ConfigMessage::GenderSelected(selection) => {
                self.gender_selection = selection;
                self.configs.default_gender_category = GENDER_CATEGORIES[selection];
            }
            ConfigMessage::SaveConfig => {
                let configs = self.configs.clone();
                return cosmic::task::future(async move {
                    if let Err(err) = write_configs(&configs) {
                        log::warn!("failed to write configs, due to {err}");
                    }
                    Message::Nop
                });
            }
            ConfigMessage::SelectedAthletesFile(athletes_file) => {
                if let Some(athletes_file) = athletes_file {
                    self.configs.athletes_file = athletes_file;
                }
            }
            ConfigMessage::SelectedClubFile(club_file) => {
                if let Some(club_file) = club_file {
                    self.configs.club_file = club_file;
                }
            }
            ConfigMessage::SelectedTournamentBasedir(tournament_basedir) => {
                if let Some(tournament_basedir) = tournament_basedir {
                    self.configs.tournament_basedir = tournament_basedir;
                }
            }
        }
        Task::none()
    }

    pub fn view_about(&self) -> Element<<Self as Application>::Message> {
        let mut column = widget::column::with_capacity(7)
            .push(widget::text(translate!("about.about", &self.translations)))
            .push(widget::divider::horizontal::default())
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxs)
                .push(widget::text(translate!("about.version", &self.translations)))
                .push(widget::text(VERSION)))
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxxs)
                .push(widget::text(translate!("about.license", &self.translations)))
                .push(widget::tooltip(widget::button::link(LICENSE)
                    .on_press(Message::About(AboutMessage::License)),
                widget::text(LICENSE_LINK),
                Position::Bottom)))
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxxs)
                .push(widget::text(translate!("about.source_code", &self.translations)))
                .push(widget::tooltip(widget::button::link(CODE_LINK)
                    .on_press(Message::About(AboutMessage::Code)),
                widget::text(CODE_LINK),
                Position::Bottom)))
            .push(widget::button::text(translate!("about.check_update", &self.translations))
                .on_press(Message::About(AboutMessage::CheckUpdate)));
        if let Some(popup_text) = &self.update_check_text {
            column = column.push(widget::dialog()
                .body(popup_text)
                .title(translate!("about.update_popup_title", &self.translations))
                .primary_action(widget::button::text(translate!("about.close_popup", &self.translations))
                    .on_press(Message::About(AboutMessage::CheckUpdateClose))));
        }
        column.into()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn update_about(&mut self, message: AboutMessage) -> Task<<Self as Application>::Message> {
        match message {
            AboutMessage::Code => {
                let _ = open::that_detached(CODE_LINK);
            }
            AboutMessage::License => {
                let _ = open::that_detached(LICENSE_LINK);
            }
            AboutMessage::CheckUpdate => {
                return cosmic::task::future(async {
                    let update_available = check_update_available(VERSION);
                    let update_text = if let Ok(update_available) = update_available {
                        match update_available {
                            UpdateAvailability::UpdateAvailable => {
                                "about.update_available"
                            }
                            UpdateAvailability::NoUpdateAvailable => {
                                "about.no_update_available"
                            }
                            UpdateAvailability::RunningUnstable => {
                                "about.running_unstable"
                            }
                        }
                    } else {
                        log::warn!("failed to get new version information from network: {}", update_available.unwrap_err());  // cannot panic as it was checked above for `Ok`
                        "about.no_network"
                    };
                    Message::About(AboutMessage::UpdateChecked(update_text))
                });
            }
            AboutMessage::CheckUpdateClose => {
                self.update_check_text = None;
            }
            AboutMessage::UpdateChecked(update_text) => {
                self.update_check_text = Some(translate!(update_text, &self.translations));
            }
        }
        Task::none()
    }
}