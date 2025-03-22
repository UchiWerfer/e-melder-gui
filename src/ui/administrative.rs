use cosmic::{widget, Application, Element};

use crate::ui::EMelderApp;

impl EMelderApp {
    pub fn view_edit_club(&self) -> Element<<Self as Application>::Message> {
        widget::text("edit-club").into()
    }

    pub fn view_config(&self) -> Element<<Self as Application>::Message> {
        widget::text("config").into()
    }

    pub fn view_about(&self) -> Element<<Self as Application>::Message> {
        widget::text("about").into()
    }
}