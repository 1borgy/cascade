use iced::{
    widget::container::{transparent, Catalog, Style, StyleFn},
    Background, Border,
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

// pub fn none(_theme: &Theme) -> Style {
//     Style {
//         background: None,
//         ..Default::default()
//     }
// }

pub fn monobox(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.secondary.scale_alpha(0.5))),
        border: Border {
            color: theme.secondary,
            width: 1.0,
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn bordered(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.background)),
        border: Border {
            color: theme.highlight,
            width: 1.5,
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
