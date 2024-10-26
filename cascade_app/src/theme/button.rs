use iced::{
    widget::button::{Catalog, Status, Style, StyleFn},
    Background, Border, Color,
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

fn default(theme: &Theme, status: Status) -> Style {
    primary(theme, status)
}

fn button(fg: Color, bg: Color, status: Status) -> Style {
    match status {
        Status::Active | Status::Pressed => Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(bg.scale_alpha(0.8))),
            text_color: fg,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Disabled => Style {
            background: Some(Background::Color(bg.scale_alpha(0.2))),
            text_color: fg.scale_alpha(0.5),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    button(theme.background, theme.primary, status)
}

pub fn secondary(theme: &Theme, status: Status) -> Style {
    button(theme.text, theme.secondary, status)
}

// pub fn warning(theme: &Theme, status: Status) -> Style {
//     button(theme.background, theme.warning, status)
// }
//
// pub fn danger(theme: &Theme, status: Status) -> Style {
//     button(theme.background, theme.danger, status)
// }

fn entry_button(bg: Color, fg: Color, accent: Color, status: Status) -> Style {
    let bg_pressed = accent.scale_alpha(0.5);
    let bg_hovered = accent.scale_alpha(0.2);

    match status {
        Status::Active => Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: 4.0.into(),
                color: accent,
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Pressed => Style {
            background: Some(Background::Color(bg_pressed)),
            text_color: fg,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: accent,
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(bg_hovered)),
            text_color: fg,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: accent,
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Disabled => Style {
            background: Some(Background::Color(bg.scale_alpha(0.2))),
            text_color: fg.scale_alpha(0.2),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

pub fn entry_selected(theme: &Theme, status: Status) -> Style {
    entry_button(
        theme.secondary.scale_alpha(0.4),
        theme.text,
        theme.primary,
        status,
    )
}

pub fn entry_unselected(theme: &Theme, status: Status) -> Style {
    entry_button(theme.background, theme.text, theme.primary, status)
}

pub fn entry_queued(theme: &Theme, status: Status) -> Style {
    entry_button(
        theme.secondary.scale_alpha(0.2),
        theme.text,
        theme.danger,
        status,
    )
}

pub fn entry_warning(theme: &Theme, status: Status) -> Style {
    entry_button(
        theme.warning.scale_alpha(0.2),
        theme.text,
        theme.warning,
        status,
    )
}

pub fn entry_success(theme: &Theme, status: Status) -> Style {
    entry_button(
        theme.success.scale_alpha(0.2),
        theme.text,
        theme.success,
        status,
    )
}

pub fn entry_danger(theme: &Theme, status: Status) -> Style {
    entry_button(
        theme.danger.scale_alpha(0.3),
        theme.text,
        theme.danger,
        status,
    )
}
