use cosmic::{widget, Application, Element};

use crate::ui::EMelderApp;

impl EMelderApp {
    pub fn view_registering(&self) -> Element<<Self as Application>::Message> {
        widget::text("registering").into()
    }

    pub fn view_adding(&self) -> Element<<Self as Application>::Message> {
        widget::text("adding").into()
    }

    pub fn view_edit_athlete(&self) -> Element<<Self as Application>::Message> {
        widget::text("edit-athlete").into()
    }

    pub fn view_deleting(&self) -> Element<<Self as Application>::Message> {
        widget::text("deleting").into()
    }
}