use iced::{widget::text, Element};

use crate::fonts;

pub fn heading<M>(label: &str) -> Element<M> {
    text(label)
        //.style(iced::theme::Text::Color(theme.palette().primary))
        .size(32)
        .font(fonts::IOSEVKA_BOLD)
        .into()
}
