use enum_iterator::all;
use iced::{
    widget::{column, pick_list, row, text},
    Element, Renderer,
};

use crate::config::{CascadeBackground, CascadeColor, CascadeTheme};

#[derive(Clone, Copy, Debug)]
pub enum ThemeMessage {
    ColorChanged(CascadeColor),
    BackgroundChanged(CascadeBackground),

    ThemeChanged(CascadeTheme),
}

pub struct ThemeComponent {
    theme: CascadeTheme,
}

impl ThemeComponent {
    pub fn new(theme: CascadeTheme) -> Self {
        Self { theme }
    }

    pub fn set_theme(&mut self, theme: CascadeTheme) {
        self.theme = theme;
    }

    pub fn update(&mut self, message: ThemeMessage) -> Option<ThemeMessage> {
        match message {
            // this is a message we only send to the parent component
            ThemeMessage::ThemeChanged(_) => None,

            ThemeMessage::ColorChanged(color) => {
                self.theme.color = color;
                Some(ThemeMessage::ThemeChanged(self.theme.clone()))
            }

            ThemeMessage::BackgroundChanged(background) => {
                self.theme.background = background;
                Some(ThemeMessage::ThemeChanged(self.theme.clone()))
            }
        }
    }

    pub fn view(&self) -> Element<ThemeMessage, Renderer> {
        row![
            column![
                text("background"),
                pick_list(
                    all::<CascadeBackground>().collect::<Vec<_>>(),
                    Some(self.theme.background),
                    ThemeMessage::BackgroundChanged
                ),
            ]
            .spacing(5),
            column![
                text("color"),
                pick_list(
                    all::<CascadeColor>().collect::<Vec<_>>(),
                    Some(self.theme.color),
                    ThemeMessage::ColorChanged
                ),
            ]
            .spacing(5)
        ]
        .spacing(10)
        .into()
    }
}
