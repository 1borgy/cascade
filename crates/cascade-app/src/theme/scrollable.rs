use iced::{
    Background, Border, Color, Shadow,
    widget::{
        container,
        scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style, StyleFn},
    },
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
            background: Background::Color(theme.text.scale_alpha(0.5)),
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
    };

    match status {
        Status::Active { .. } | Status::Hovered { .. } | Status::Dragged { .. } => Style {
            container: container::Style {
                ..Default::default()
            },
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(theme.text.scale_alpha(0.5)),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: theme.secondary.scale_alpha(0.5),
            },
        },
    }
}
