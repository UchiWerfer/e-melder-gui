use cosmic::{theme, widget, Application, Element};
use cosmic::app::Task;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::Length;
use cosmic::prelude::CollectionWidget;
use crate::tournament_info::{Athlete, WeightCategory};
use crate::translate;
use crate::ui::app::Message;
use crate::ui::EMelderApp;
use crate::utils::{write_athletes, BELTS, GENDERS};

#[derive(Clone, Debug)]
pub enum RegisteringMessage {
}

#[derive(Clone, Debug)]
pub enum AddingMessage {
    GivenName(String),
    SurName(String),
    BeltSelection(usize),
    BirthYear(String),
    GenderSelection(usize),
    Add
}

#[derive(Clone, Debug)]
pub enum EditAthleteMessage {
    GivenName(String, usize),
    SurName(String, usize),
    Gender(usize, usize),
    Graduate(usize)
}

pub type DeletingMessage = usize;

impl EMelderApp {
    pub fn view_registering(&self) -> Element<<Self as Application>::Message> {
        widget::text("registering").into()
    }

    pub fn update_registering(&mut self, message: RegisteringMessage) -> Task<<Self as Application>::Message> {
        Task::none()
    }

    pub fn view_adding(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(6)
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("add.given_name", &self.translations)))
                .push(widget::text_input("", &self.adding.given_name)
                    .on_input(|input| Message::Adding(AddingMessage::GivenName(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("add.sur_name", &self.translations)))
                .push(widget::text_input("", &self.adding.sur_name)
                    .on_input(|input| Message::Adding(AddingMessage::SurName(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::dropdown(&self.belt_names,
                Some(self.belt_selection),
                |selection| Message::Adding(AddingMessage::BeltSelection(selection))))
                .push(widget::text(translate!("add.belt", &self.translations))))
            .push(widget::row::with_capacity(2)
                .push(widget::text(translate!("add.year", &self.translations)))
                .push(widget::text_input("", self.adding.year.to_string())
                    .on_input(|input| Message::Adding(AddingMessage::BirthYear(input)))))
            .push(widget::row::with_capacity(2)
                .push(widget::dropdown(&self.genders,
                Some(self.adding_gender_selection),
                |selection| Message::Adding(AddingMessage::GenderSelection(selection))))
                .push(widget::text(translate!("add.gender", &self.translations))))
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
            }
            AddingMessage::GenderSelection(selection) => {
                self.adding_gender_selection = selection;
            }
            AddingMessage::Add => {
                let belt = BELTS[self.belt_selection];
                let gender = GENDERS[self.adding_gender_selection];
                self.athletes.push(Athlete::new(
                    self.adding.given_name.clone(), self.adding.sur_name.clone(),
                    self.adding.year, belt, WeightCategory::default(), gender
                ));
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
        widget::column::with_capacity(self.athletes.len())
            .extend(self.athletes.iter().enumerate().map(|(index, athlete)| {
                widget::row::with_capacity(5 + <bool as Into<usize>>::into(athlete.get_belt().upgradable()))
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
        }
        let athletes_file = self.configs.athletes_file.clone();
        let athletes =  self.athletes.clone();
        cosmic::task::future(async move {
            if let Err(err) = write_athletes(&athletes_file, &athletes) {
                log::warn!("failed to write athletes, due to {err}");
            }
            Message::Nop
        })
    }

    pub fn view_deleting(&self) -> Element<<Self as Application>::Message> {
        widget::column::with_capacity(self.athletes.len())
            .extend(self.athletes.iter().enumerate().map(|(index, athlete)| {
                widget::row::with_capacity(6)
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
            .into()
    }

    pub fn update_deleting(&mut self, message: DeletingMessage) -> Task<<Self as Application>::Message> {
        self.athletes.remove(message);
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