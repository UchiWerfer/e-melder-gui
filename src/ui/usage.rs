use cosmic::{widget, Application, Element};
use cosmic::app::Task;
use crate::ui::EMelderApp;

#[derive(Clone, Debug)]
pub enum RegisteringMessage {
}

#[derive(Clone, Debug)]
pub enum AddingMessage {
}

#[derive(Clone, Debug)]
pub enum EditAthleteMessage {
}

#[derive(Clone, Debug)]
pub enum DeletingMessage {
}

impl EMelderApp {
    pub fn view_registering(&self) -> Element<<Self as Application>::Message> {
        widget::text("registering").into()
    }

    pub fn update_registering(&mut self, message: RegisteringMessage) -> Task<<Self as Application>::Message> {
        Task::none()
    }

    pub fn view_adding(&self) -> Element<<Self as Application>::Message> {
        widget::text("adding").into()
    }

    pub fn update_adding(&mut self, message: AddingMessage) -> Task<<Self as Application>::Message> {
        Task::none()
    }

    pub fn view_edit_athlete(&self) -> Element<<Self as Application>::Message> {
        widget::text("edit-athlete").into()
    }

    pub fn update_edit_athlete(&mut self, message: EditAthleteMessage) -> Task<<Self as Application>::Message> {
        Task::none()
    }

    pub fn view_deleting(&self) -> Element<<Self as Application>::Message> {
        widget::text("deleting").into()
    }

    pub fn update_deleting(&mut self, message: DeletingMessage) -> Task<<Self as Application>::Message> {
        Task::none()
    }
}