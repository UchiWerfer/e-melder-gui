use chrono::NaiveDate;
use cosmic::{theme, widget, Application, Apply, Element};
use cosmic::app::Task;
use cosmic::iced::{Length, Pixels};
use cosmic::iced::alignment::Vertical;
use cosmic::widget::button::Builder;
use cosmic::widget::Text;

use crate::tournament_info::{registering_athletes_to_tournaments, Athlete, RegisteringAthlete, WeightCategory};
use crate::translate;
use crate::ui::app::Message;
use crate::ui::EMelderApp;
use crate::utils::{write_athletes, write_tournaments, BELTS, GENDERS, LEGAL_GENDER_CATEGORIES};

#[derive(Clone, Debug)]
pub enum RegisteringMessage {
    Name(String),
    Place(String),
    Date(NaiveDate),
    Register,
    Search(String),
    Add(usize),
    GenderCategory(usize, usize),
    AgeCategory(String, usize),
    WeightCategory(String, usize),
    Delete(usize),
    ToggleDate,
    NextMonth,
    PrevMonth
}

#[derive(Clone, Debug)]
pub enum AddingMessage {
    GivenName(String),
    SurName(String),
    BeltSelection(usize),
    BirthYear(String),
    GenderSelection(usize),
    WeightCategory(String),
    Add
}

#[derive(Clone, Debug)]
pub enum EditAthleteMessage {
    GivenName(String, usize),
    SurName(String, usize),
    Gender(usize, usize),
    Graduate(usize),
    WeightCategory(String, usize),
    Commit
}

pub type DeletingMessage = usize;

impl EMelderApp {
    #[allow(clippy::too_many_lines)]
    pub fn view_registering(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(9)
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("register.name", &self.translations)))
                .push(widget::text_input("", &self.registering.name)
                    .on_input(|input| Message::Registering(RegisteringMessage::Name(input)))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("register.place", &self.translations)))
                .push(widget::text_input("", &self.registering.place)
                    .on_input(|input| Message::Registering(RegisteringMessage::Place(input)))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("register.date", &self.translations)))
                .push(widget::button::text(self.registering.date.format("%d.%m.%Y").to_string())
                    .on_press(Message::Registering(RegisteringMessage::ToggleDate))))
            .push_maybe(if self.show_date {
                Some(widget::popover(widget::calendar(&self.calendar_model,
                |date| Message::Registering(RegisteringMessage::Date(date)),
                || Message::Registering(RegisteringMessage::PrevMonth),
                                                      || Message::Registering(RegisteringMessage::NextMonth))))
            }
            else {
                None
            })
            .push(widget::button::text(translate!("register.register", &self.translations))
                .on_press(Message::Registering(RegisteringMessage::Register)))
            .push(widget::divider::horizontal::heavy())
            .push(widget::column::with_capacity(2)
                .push(widget::row::with_capacity(2)
                    .align_y(Vertical::Center)
                    .push(widget::text(translate!("register.search", &self.translations)))
                    .push(widget::text_input::search_input("", &self.registering.search)
                        .on_input(|input| Message::Registering(RegisteringMessage::Search(input)))))
                .push(widget::container(widget::column::with_capacity(self.athletes.len())
                    .extend(self.athletes.iter().enumerate().filter(|(_, athlete)| matches_query(
                        &format!("{} {}", athlete.get_given_name(), athlete.get_sur_name()),
                        &self.registering.search))
                        .map(|(index, athlete)| {
                            widget::row::with_capacity(6)
                                .align_y(Vertical::Center)
                                .push(widget::text(athlete.get_given_name())
                                    .width(Length::Fixed(100.0)))
                                .push(widget::text(athlete.get_sur_name())
                                    .width(Length::Fixed(100.0)))
                                .push(widget::text(translate!(&format!("register.table.gender_category.{}",
                                athlete.get_gender().render()), &self.translations))
                                    .width(Length::Fixed(80.0)))
                                .push(widget::text(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()),
                                &self.translations))
                                    .width(Length::Fixed(150.0)))
                                .push(widget::text(athlete.get_birth_year().to_string())
                                    .width(Length::Fixed(40.0)))
                                .push(widget::button::text(translate!("register.table.add", &self.translations))
                                    .on_press_maybe(if self.registering.athletes.iter().any(|reg_athlete| reg_athlete.index == index) {
                                        None
                                    }
                                    else {
                                        Some(Message::Registering(RegisteringMessage::Add(index)))
                                    }))
                                .into()
                        }))
                    .push_maybe(if self.athletes.iter().any(|athlete| matches_query(
                        &format!("{} {}", athlete.get_given_name(), athlete.get_sur_name()),
                        &self.registering.search
                    )) {
                        None
                    }
                    else {
                        Some(widget::text(translate!("register.search.empty", &self.translations)))
                    })
                    .apply(widget::scrollable))
                    .max_height(Pixels(200.0))))
            .push(widget::divider::horizontal::default())
            .push(widget::column::with_capacity(self.registering.athletes.len() + 1)
                .push(widget::row::with_capacity(7)
                    .align_y(Vertical::Center)
                    .push(widget::text::title4(translate!("register.table.given_name", &self.translations))
                        .width(Length::Fixed(120.0)))
                    .push(widget::text::title4(translate!("register.table.sur_name", &self.translations))
                        .width(Length::Fixed(120.0)))
                    .push(widget::text::title4(translate!("register.table.belt", &self.translations))
                        .width(Length::Fixed(150.0)))
                    .push(widget::text::title4(translate!("register.table.year", &self.translations))
                        .width(Length::Fixed(130.0)))
                    .push(widget::text::title4(translate!("register.table.gender_category", &self.translations))
                        .width(Length::Fixed(200.0)))
                    .push(widget::text::title4(translate!("register.table.age_category", &self.translations))
                        .width(Length::Fixed(150.0)))
                    .push(widget::text::title4(translate!("register.table.weight_category", &self.translations))
                        .width(Length::Fixed(160.0))))
                .extend(self.registering.athletes.iter().enumerate()
                    .map(|(index, athlete)| {
                        widget::row::with_capacity(8)
                            .align_y(Vertical::Center)
                            .push(widget::text(athlete.get_given_name())
                                .width(Length::Fixed(120.0)))
                            .push(widget::text(athlete.get_sur_name())
                                .width(Length::Fixed(120.0)))
                            .push(widget::text(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()),
                            &self.translations))
                                .width(Length::Fixed(150.0)))
                            .push(widget::text(athlete.get_birth_year().to_string())
                                .width(Length::Fixed(130.0)))
                            .push(widget::dropdown(&self.legal_gender_categories[athlete.get_gender()],
                                                   LEGAL_GENDER_CATEGORIES[athlete.get_gender()].iter().position(|gender| {
                                                       *gender == athlete.get_gender_category()
                                                   }),
                                                   move |selection| Message::Registering(RegisteringMessage::GenderCategory(selection, index))
                            )
                                .width(Length::Fixed(200.0)))
                            .push(widget::text_input("", athlete.get_age_category())
                                .on_input(move |input| Message::Registering(RegisteringMessage::AgeCategory(input, index)))
                                .width(Length::Fixed(150.0)))
                            .push(widget::text_input("", athlete.get_weight_category())
                                .on_input(move |input| Message::Registering(RegisteringMessage::WeightCategory(input, index)))
                                .width(Length::Fixed(160.0)))
                            .push(widget::button::text(translate!("register.table.delete", &self.translations))
                                .on_press(Message::Registering(RegisteringMessage::Delete(index))))
                            .into()
                    }))
                .push_maybe(if self.registering.athletes.is_empty() {
                    Some(widget::text(translate!("register.table.empty", &self.translations)))
                }
                else {
                    None
                })
                .apply(widget::scrollable))
            .into()
    }

    #[allow(clippy::too_many_lines)]
    pub fn update_registering(&mut self, message: RegisteringMessage) -> Task<<Self as Application>::Message> {
        match message {
            RegisteringMessage::Name(name) => {
                self.registering.name = name;
            }
            RegisteringMessage::Place(place) => {
                self.registering.place = place;
            }
            RegisteringMessage::Date(date) => {
                self.registering.date = date;
                self.calendar_model.selected = date;
            }
            RegisteringMessage::Register => {
                let reg_athletes = self.registering.athletes.clone();
                for reg_athlete in &self.registering.athletes {
                    if let Some(weight_category) = WeightCategory::from_str(reg_athlete.get_weight_category()) {
                        *self.athletes[reg_athlete.index].get_weight_category_mut() = weight_category;
                        self.edit_athlete_weight_categories[reg_athlete.index] = weight_category.to_string();
                    }
                    else {
                        let translations = self.translations.clone();
                        std::thread::spawn(move || {
                            #[cfg(all(target_family="unix", not(target_os="macos")))]
                            let _ = notify_rust::Notification::new()
                                .summary(&translate!("application.title", &translations))
                                .body(&translate!("register.notification.invalid_weight_category", &translations))
                                .sound_name("dialog-error")
                                .show().map(|handle| handle.wait_for_action(|_| {}));
                            #[cfg(not(all(target_family="unix", not(target_os="macos"))))]
                            let _ = notify_rust::Notification::new()
                                .summary(&translate!("application.title", &translations))
                                .body(&translate!("register.notification.invalid_weight_category", &translations))
                                .show();
                        });
                        return Task::none();
                    }
                }
                let athletes = self.athletes.clone();
                let name = self.registering.name.clone();
                let place = self.registering.place.clone();
                let club = self.club.clone();
                let configs = self.configs.clone();
                let date = self.registering.date;
                let translations = self.translations.clone();
                let athletes_file = self.configs.athletes_file.clone();
                return cosmic::task::future(async move {
                    let tournaments = registering_athletes_to_tournaments(&reg_athletes,
                                                                          &name, date, &place, &club).expect("checked for this implicitly above");

                    match write_tournaments(&tournaments, &configs) {
                        Ok(()) => {
                            let tournament_basedir = configs.tournament_basedir.clone();
                            #[cfg(all(target_family="unix", not(target_os="macos")))]
                            std::thread::spawn(move || {
                                let _ = notify_rust::Notification::new()
                                    .summary(&translate!("application.title", &translations))
                                    .body(&translate!("register.notification.ask", &translations))
                                    .sound_name("dialog-question")
                                    .action("yes", &translate!("register.notification.yes", &translations))
                                    .action("no", &translate!("register.notification.no", &translations))
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
                        Err(err) => {
                            log::warn!("failed to write tournaments, due to {err}");
                            std::thread::spawn(move || {
                                #[cfg(all(target_family="unix", not(target_os="macos")))]
                                let _ = notify_rust::Notification::new()
                                    .summary(&translate!("application.title", &translations))
                                    .body(&translate!("register.notification.io_error", &translations))
                                    .sound_name("dialog-error")
                                    .show().map(|handle| handle.wait_for_action(|_| {}));
                                #[cfg(not(all(target_family="unix", not(target_os="macos"))))]
                                let _ = notify_rust::Notification::new()
                                    .summary(&translate!("application.title", &translations))
                                    .body(&translate!("register.notification.io_error", &translations))
                                    .show();
                            });
                        }
                    }

                    if let Err(err) = write_athletes(&athletes_file, &athletes) {
                        log::warn!("failed to write athletes due to {err}");
                    }

                    Message::Nop
                });
            }
            RegisteringMessage::Search(search) => {
                self.registering.search = search;
            }
            RegisteringMessage::Add(selection) => {
                if self.registering.athletes.iter().any(|reg_athlete| reg_athlete.index == selection) {
                    return Task::none();
                }
                self.registering.athletes.push(RegisteringAthlete::from_athlete(&self.athletes[selection], selection));
            }
            RegisteringMessage::GenderCategory(selection, index) => {
                let gender = self.registering.athletes[index].get_gender();
                *self.registering.athletes[index].get_gender_category_mut() = LEGAL_GENDER_CATEGORIES[gender][selection];
            }
            RegisteringMessage::AgeCategory(age_category, index) => {
                self.registering.athletes[index].get_age_category_mut().clone_from(&age_category);
            }
            RegisteringMessage::WeightCategory(weight_category, index) => {
                let trimmed = weight_category.replace(char::is_whitespace, "");
                if trimmed.is_empty() || (trimmed.starts_with(['+', '-', '0', '1', '2', '3', '4',
                    '5', '6', '7', '8', '9']) && trimmed.chars().skip(1).all(char::is_numeric)) {
                    self.registering.athletes[index].get_weight_category_mut().clone_from(&trimmed);
                }
            }
            RegisteringMessage::Delete(index) => {
                self.registering.athletes.remove(index);
            }
            RegisteringMessage::ToggleDate => {
                self.show_date = !self.show_date;
            }
            RegisteringMessage::NextMonth => {
                self.calendar_model.show_next_month();
            }
            RegisteringMessage::PrevMonth => {
                self.calendar_model.show_prev_month();
            }
        }
        Task::none()
    }

    pub fn view_adding(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(7)
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("add.given_name", &self.translations)))
                .push(widget::text_input("", &self.adding.given_name)
                    .on_input(|input| Message::Adding(AddingMessage::GivenName(input)))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("add.sur_name", &self.translations)))
                .push(widget::text_input("", &self.adding.sur_name)
                    .on_input(|input| Message::Adding(AddingMessage::SurName(input)))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::dropdown(&self.belt_names,
                Some(self.belt_selection),
                |selection| Message::Adding(AddingMessage::BeltSelection(selection))))
                .push(widget::text(translate!("add.belt", &self.translations))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("add.year", &self.translations)))
                .push(widget::text_input("", self.adding.year.to_string())
                    .on_input(|input| Message::Adding(AddingMessage::BirthYear(input)))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::dropdown(&self.genders,
                Some(self.adding_gender_selection),
                |selection| Message::Adding(AddingMessage::GenderSelection(selection))))
                .push(widget::text(translate!("add.gender", &self.translations))))
            .push(widget::row::with_capacity(2)
                .align_y(Vertical::Center)
                .push(widget::text(translate!("add.weight_category", &self.translations)))
                .push(widget::text_input("", &self.adding.weight_category)
                    .on_input(|input| Message::Adding(AddingMessage::WeightCategory(input)))))
            .push(widget::button::text(translate!("add.commit", &self.translations))
                .on_press(Message::Adding(AddingMessage::Add)))
            .into()
    }

    pub fn update_adding(&mut self, message: AddingMessage) -> Task<<Self as Application>::Message> {
        match message {
            AddingMessage::GivenName(given_name) => {
                self.adding.given_name = given_name;
            }
            AddingMessage::SurName(sur_name) => {
                self.adding.sur_name = sur_name;
            }
            AddingMessage::BeltSelection(selection) => {
                self.belt_selection = selection;
            }
            AddingMessage::BirthYear(birth_year_str) => {
                if let Ok(birth_year) = birth_year_str.parse() {
                    self.adding.year = birth_year;
                }
                else if birth_year_str == String::new() {
                    self.adding.year = 0;
                }
            }
            AddingMessage::GenderSelection(selection) => {
                self.adding_gender_selection = selection;
            }
            AddingMessage::WeightCategory(weight_category) => {
                let trimmed = weight_category.replace(char::is_whitespace, "");
                if trimmed.is_empty() || (trimmed.starts_with(['+', '-', '0', '1', '2', '3', '4',
                '5', '6', '7', '8', '9']) && trimmed.chars().skip(1).all(char::is_numeric)) {
                    self.adding.weight_category = trimmed;
                }
            }
            AddingMessage::Add => {
                let belt = BELTS[self.belt_selection];
                let gender = GENDERS[self.adding_gender_selection];
                let Some(weight_category) = WeightCategory::from_str(&self.adding.weight_category) else {
                    return Task::none();
                };
                self.athletes.push(Athlete::new(
                    self.adding.given_name.clone(), self.adding.sur_name.clone(),
                    self.adding.year, belt, weight_category, gender
                ));
                self.edit_athlete_weight_categories.push(weight_category.to_string());
                self.adding.clear();
                self.adding_gender_selection = self.gender_selection;
                self.belt_selection = 0;
                let athletes_file = self.configs.athletes_file.clone();
                let athletes = self.athletes.clone();
                return cosmic::task::future(async move {
                    if let Err(err) = write_athletes(&athletes_file, &athletes) {
                        log::warn!("failed to write athletes, due to {err}");
                    }
                    Message::Nop
                });
            }
        }
        Task::none()
    }

    pub fn view_edit_athlete(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(self.athletes.len() + 1)
            .extend(self.athletes.iter().enumerate().map(|(index, athlete)| {
                widget::row::with_capacity(6 + <bool as Into<usize>>::into(athlete.get_belt().upgradable()))
                    .align_y(Vertical::Center)
                    .spacing(theme::active().cosmic().spacing.space_xs)
                    .push(widget::text_input("", athlete.get_given_name())
                        .on_input(move |input| Message::EditAthlete(EditAthleteMessage::GivenName(input, index)))
                        .width(Length::Fixed(150.0)))
                    .push(widget::text_input("", athlete.get_sur_name())
                        .on_input(move |input| Message::EditAthlete(EditAthleteMessage::SurName(input, index)))
                        .width(Length::Fixed(150.0)))
                    .push(widget::text(athlete.get_birth_year().to_string()).width(Length::Fixed(40.0)))
                    .push(widget::dropdown(&self.genders,
                    GENDERS.iter().position(|gender| {
                        *gender == athlete.get_gender()
                    }),
                    move |selection| Message::EditAthlete(EditAthleteMessage::Gender(selection, index)))
                        .width(Length::Fixed(80.0)))
                    .push(widget::text_input("", &self.edit_athlete_weight_categories[index])
                        .on_input(move |input| Message::EditAthlete(EditAthleteMessage::WeightCategory(input, index)))
                        .width(Length::Fixed(80.0)))
                    .push(widget::text(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()), &self.translations))
                        .width(Length::Fixed(150.0)))
                    .push_maybe(if athlete.get_belt().upgradable() {
                        Some(widget::button::text(translate!("edit_athlete.graduate", &self.translations))
                            .on_press(Message::EditAthlete(EditAthleteMessage::Graduate(index))))
                    }
                    else {
                        None
                    })
                    .into()
            }))
            .push(if self.athletes.is_empty() {
                <Text<_> as Into<Element<_>>>::into(widget::text(translate!("edit_athlete.empty", &self.translations)))
            }
            else {
                <Builder<_, _> as Into<Element<_>>>::into(widget::button::text(translate!("edit_athlete.commit", &self.translations))
                    .on_press(Message::EditAthlete(EditAthleteMessage::Commit)))
            })
            .into()
    }

    pub fn update_edit_athlete(&mut self, message: EditAthleteMessage) -> Task<<Self as Application>::Message> {
        match message {
            EditAthleteMessage::GivenName(given_name, index) => {
                self.athletes[index].get_given_name_mut().clone_from(&given_name);
            }
            EditAthleteMessage::SurName(sur_name, index) => {
                self.athletes[index].get_sur_name_mut().clone_from(&sur_name);
            }
            EditAthleteMessage::Gender(selection, index) => {
                *self.athletes[index].get_gender_mut() = GENDERS[selection];
            }
            EditAthleteMessage::Graduate(index) => {
                let belt = self.athletes[index].get_belt();
                *self.athletes[index].get_belt_mut() = belt.inc();
            }
            EditAthleteMessage::WeightCategory(weight_category, index) => {
                let trimmed = weight_category.replace(char::is_whitespace, "");
                if trimmed.is_empty() || (trimmed.starts_with(['+', '-', '0', '1', '2', '3', '4',
                '5', '6', '7', '8', '9']) && trimmed.chars().skip(1).all(char::is_numeric)) {
                    self.edit_athlete_weight_categories[index] = trimmed;
                }
            }
            EditAthleteMessage::Commit => {
                for (athlete, weight_category_str) in self.athletes.iter_mut().zip(self.edit_athlete_weight_categories.iter()) {
                    if let Some(weight_category) = WeightCategory::from_str(weight_category_str) {
                        *athlete.get_weight_category_mut() = weight_category;
                    }
                }
                let athletes_file = self.configs.athletes_file.clone();
                let athletes =  self.athletes.clone();
                return cosmic::task::future(async move {
                    if let Err(err) = write_athletes(&athletes_file, &athletes) {
                        log::warn!("failed to write athletes, due to {err}");
                    }
                    Message::Nop
                });
            }
        }
        Task::none()
    }

    pub fn view_deleting(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(self.athletes.len())
            .extend(self.athletes.iter().enumerate().map(|(index, athlete)| {
                widget::row::with_capacity(6)
                    .align_y(Vertical::Center)
                    .push(widget::text(athlete.get_given_name()).width(Length::Fixed(100.0)))
                    .push(widget::text(athlete.get_sur_name()).width(Length::Fixed(100.0)))
                    .push(widget::text(athlete.get_birth_year().to_string()).width(Length::Fixed(40.0)))
                    .push(widget::text(translate!(&format!("register.table.gender_category.{}",
                        athlete.get_gender().render()), &self.translations)).width(Length::Fixed(80.0)))
                    .push(widget::text(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()),
                    &self.translations)).width(Length::Fixed(150.0)))
                    .push(widget::button::text(translate!("delete.delete", &self.translations))
                        .on_press(Message::Deleting(index)))
                    .into()
            }))
            .push_maybe(if self.athletes.is_empty() {
                Some(widget::text(translate!("delete.empty", &self.translations)))
            }
            else {
                None
            })
            .into()
    }

    pub fn update_deleting(&mut self, message: DeletingMessage) -> Task<<Self as Application>::Message> {
        self.athletes.remove(message);
        self.edit_athlete_weight_categories.remove(message);

        if let Some(registering_to_delete) = self.registering.athletes.iter().position(|reg_athlete| reg_athlete.index == message) {
            self.registering.athletes.remove(registering_to_delete);
        }

        for reg_athlete in &mut self.registering.athletes {
            if reg_athlete.index > message {
                reg_athlete.index -= 1;
            }
        }

        let athletes_file = self.configs.athletes_file.clone();
        let athletes = self.athletes.clone();
        cosmic::task::future(async move {
            if let Err(err) = write_athletes(&athletes_file, &athletes) {
                log::warn!("failed to write athletes, due to {err}");
            }
            Message::Nop
        })
    }
}

fn matches_query(base: &str, query: &str) -> bool {
    // value for comparison was obtained by testing various values and choosing
    // the values with the results that felt best
    base.contains(query) || textdistance::nstr::jaro(base, query) >= 0.65
}
