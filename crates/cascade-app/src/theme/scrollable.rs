use iced::{
    widget::{
        container,
        scrollable::{Catalog, Rail, Scroller, Status, Style, StyleFn},
    },
    Background, Border, Color,
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
    let rail = Rail {
        background: Some(Background::Color(theme.secondary.scale_alpha(0.5))),
        border: Border::default(),
        scroller: Scroller {
            color: theme.text.scale_alpha(0.5),
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
    };

    match status {
        Status::Active | Status::Hovered { .. } | Status::Dragged { .. } => Style {
            container: container::Style {
                ..Default::default()
            },
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
        },
    }
}
