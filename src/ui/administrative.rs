use cosmic::{theme, widget, Application, Element};
use cosmic::widget::tooltip::Position;

use crate::translate;
use crate::ui::app::Message;
use crate::ui::EMelderApp;
use crate::utils::{CODE_LINK, LICENSE, LICENSE_LINK, VERSION};

impl EMelderApp {
    pub fn view_edit_club(&self) -> Element<<Self as Application>::Message> {
        widget::text("edit-club").into()
    }

    pub fn view_config(&self) -> Element<<Self as Application>::Message> {
        widget::text("config").into()
    }

    pub fn view_about(&self) -> Element<<Self as Application>::Message> {
        let mut column = widget::column::with_capacity(6)
            .push(widget::text(translate!("about.about", &self.translations)))
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxs)
                .push(widget::text(translate!("about.version", &self.translations)))
                .push(widget::text(VERSION)))
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxxs)
                .push(widget::text(translate!("about.license", &self.translations)))
                .push(widget::tooltip(widget::button::link(LICENSE)
                    .on_press(Message::License),
                widget::text(LICENSE_LINK),
                Position::Bottom)))
            .push(widget::row::with_capacity(2)
                .spacing(theme::active().cosmic().spacing.space_xxxs)
                .push(widget::text(translate!("about.source_code", &self.translations)))
                .push(widget::tooltip(widget::button::link(CODE_LINK)
                    .on_press(Message::Code),
                widget::text(CODE_LINK),
                Position::Bottom)))
            .push(widget::button::text(translate!("about.check_update", &self.translations))
                .on_press(Message::CheckUpdate));
        if let Some(popup_text) = &self.update_check_text {
            column = column.push(widget::dialog()
                .body(popup_text)
                .title(translate!("about.update_popup_title", &self.translations))
                .primary_action(widget::button::text(translate!("about.close_popup", &self.translations))
                    .on_press(Message::CheckUpdateClose)));
        }
        column.into()
    }
}