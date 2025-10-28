use iced::{
    widget::pick_list::{Catalog, Status, Style, StyleFn},
    Background, Border, Color,
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>, status: Status) -> Style {
        class(self, status)
    }
}

fn default(theme: &Theme, status: Status) -> Style {
    primary(theme, status)
}

fn primary(theme: &Theme, status: Status) -> Style {
    let active = Style {
        background: Background::Color(theme.background),
        text_color: theme.text,
        placeholder_color: theme.text.scale_alpha(0.5),
        handle_color: theme.primary,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.secondary,
        },
    };

    match status {
        Status::Active | Status::Hovered | Status::Opened { .. } => active,
    }
}
