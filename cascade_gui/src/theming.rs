use cascade::config::{CascadeTheme, RgbColor};

// TODO: config should probably just go in cascade_gui
pub fn config_to_iced_color(config_color: RgbColor) -> iced::Color {
    iced::Color::from_rgb8(
        config_color.red,
        config_color.green,
        config_color.blue,
    )
}

pub fn config_to_primary_color(theme: &CascadeTheme) -> iced::Color {
    let palette = theme.background.get_palette();
    let color = palette.get_color(theme.color);
    config_to_iced_color(color)
}

pub fn config_to_subtext_color(theme: &CascadeTheme) -> iced::Color {
    let palette = theme.background.get_palette();
    config_to_iced_color(palette.subtext)
}

pub fn config_to_iced_theme(theme: &CascadeTheme) -> iced::Theme {
    let palette = theme.background.get_palette();
    let primary = palette.get_color(theme.color);

    iced::Theme::custom(iced::theme::Palette {
        background: config_to_iced_color(palette.background),
        text: config_to_iced_color(palette.text),
        primary: config_to_iced_color(primary),
        danger: config_to_iced_color(palette.red),
        success: config_to_iced_color(palette.green),
    })
}
