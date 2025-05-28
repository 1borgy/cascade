use iced::{
    widget::checkbox::{Catalog, Status, Style, StyleFn},
    Background, Border, Color,
};

use crate::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: iced::widget::checkbox::Status) -> Style {
        class(self, status)
    }
}

fn checkbox(fg: Color, bg: Color, accent: Color, status: Status) -> Style {
    let accent_hovered = accent.scale_alpha(0.7);
    let accent_disabled = accent.scale_alpha(0.2);
    let fg_disabled = fg.scale_alpha(0.5);

    let border_radius = 4.0;

    match status {
        Status::Active { is_checked } => Style {
            background: Background::Color(if is_checked { accent } else { bg }),
            icon_color: bg,
            border: Border {
                color: accent,
                width: 1.0,
                radius: border_radius.into(),
            },
            text_color: Some(fg),
        },
        Status::Hovered { .. } => Style {
            background: Background::Color(accent_hovered),
            icon_color: bg,
            border: Border {
                color: accent_hovered,
                width: 1.0,
                radius: border_radius.into(),
            },
            text_color: Some(fg),
        },
        Status::Disabled { is_checked } => Style {
            background: Background::Color(if is_checked { accent_disabled } else { bg }),
            icon_color: bg,
            border: Border {
                color: accent_disabled,
                width: 1.0,
                radius: border_radius.into(),
            },
            text_color: Some(fg_disabled),
        },
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    checkbox(theme.text, theme.background, theme.primary, status)
}

// pub fn secondary(theme: &Theme, status: Status) -> Style {
//     checkbox(theme.text, theme.background, theme.secondary, status)
// }

/// checkbox for entries
fn entry_checkbox(bg: Color, fg: Color, border: Color, status: Status) -> Style {
    let fg_disabled = fg.scale_alpha(0.6);
    let bg_disabled = bg.scale_alpha(0.3);
    let border_disabled = border.scale_alpha(0.3);

    match status {
        Status::Active { .. } | Status::Hovered { .. } => Style {
            background: Background::Color(bg),
            icon_color: fg,
            border: Border {
                color: border,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(fg),
        },
        Status::Disabled { .. } => Style {
            background: Background::Color(bg_disabled),
            icon_color: fg_disabled,
            border: Border {
                color: border_disabled,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(fg),
        },
    }
}

pub fn entry_selected(theme: &Theme, status: Status) -> Style {
    entry_checkbox(theme.primary, theme.background, theme.primary, status)
}

pub fn entry_unselected(theme: &Theme, status: Status) -> Style {
    entry_checkbox(theme.background, theme.primary, theme.primary, status)
}
