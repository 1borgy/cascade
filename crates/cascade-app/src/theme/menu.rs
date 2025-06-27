pub use iced::widget::overlay::menu::Style;
use iced::{
    widget::overlay::menu::{Catalog, StyleFn},
    Background, Border,
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(primary)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

pub fn primary(theme: &Theme) -> Style {
    Style {
        text_color: theme.text,
        background: Background::Color(theme.background),
        border: Border {
            width: 1.0,
            radius: 4.0.into(),
            color: theme.secondary,
        },
        selected_text_color: theme.text,
        selected_background: Background::Color(theme.secondary.scale_alpha(0.4)),
    }
}
