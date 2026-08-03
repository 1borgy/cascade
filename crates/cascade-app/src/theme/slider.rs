use iced::{
    Background, Border,
    widget::slider::{Catalog, Handle, HandleShape, Rail, Status, Style, StyleFn},
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active | Status::Hovered | Status::Dragged => Style {
            rail: Rail {
                backgrounds: (
                    Background::Color(theme.primary),
                    Background::Color(theme.background),
                ),
                width: 5.,
                border: Border {
                    color: theme.secondary,
                    width: 5.,
                    radius: 8.0.into(),
                },
            },
            handle: Handle {
                shape: HandleShape::Circle { radius: 8.0_f32 },
                background: Background::Color(theme.background),
                border_width: 8.0_f32,
                border_color: theme.primary,
            },
        },
    }
}
