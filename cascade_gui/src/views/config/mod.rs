use iced::{
    widget::{column, scrollable, text},
    Element, Renderer,
};

use self::{paths::PathsComponent, theme::ThemeComponent};
use crate::{
    config::CascadeConfig,
    theming::config_to_primary_color,
    views::{
        config::{paths::PathsMessage, theme::ThemeMessage},
        View,
    },
};

mod paths;
mod theme;

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Paths(PathsMessage),
    Theme(ThemeMessage),

    ConfigChanged(CascadeConfig),
}

pub struct ConfigView {
    config: CascadeConfig,

    paths: PathsComponent,
    theme: ThemeComponent,

    primary_color: iced::Color,
}

impl ConfigView {
    pub fn new(config: CascadeConfig) -> Self {
        let primary_color = config_to_primary_color(&config.theme);

        let paths = config.paths.clone();
        let theme = config.theme.clone();

        Self {
            config,
            paths: PathsComponent::new(paths),
            theme: ThemeComponent::new(theme),
            primary_color,
        }
    }
}

impl View for ConfigView {
    type Message = ConfigMessage;

    fn title(&self) -> String {
        "config".to_string()
    }

    fn set_config(&mut self, config: CascadeConfig) {
        self.primary_color = config_to_primary_color(&config.theme);

        self.paths.set_paths(config.paths.clone());
        self.theme.set_theme(config.theme.clone());

        self.config = config;
    }

    fn update(&mut self, message: ConfigMessage) -> Option<ConfigMessage> {
        match message {
            ConfigMessage::Paths(message) => {
                self.paths.update(message).map(|message| match message {
                    PathsMessage::PathsChanged(paths) => {
                        self.config.paths = paths;
                        ConfigMessage::ConfigChanged(self.config.clone())
                    }
                    _ => ConfigMessage::Paths(message),
                })
            }

            ConfigMessage::Theme(message) => {
                self.theme.update(message).map(|message| match message {
                    ThemeMessage::ThemeChanged(theme) => {
                        self.config.theme = theme;
                        ConfigMessage::ConfigChanged(self.config.clone())
                    }
                    _ => ConfigMessage::Theme(message),
                })
            }

            _ => None,
        }
    }

    fn view<'a>(&'a self) -> Element<ConfigMessage, Renderer> {
        let paths: Element<_> = column![
            text("paths")
                .size(50)
                .style(iced::theme::Text::Color(self.primary_color)),
            self.paths
                .view()
                .map(|message| ConfigMessage::Paths(message)),
        ]
        .spacing(10)
        .into();

        let theme: Element<_> = column![
            text("theme")
                .size(50)
                .style(iced::theme::Text::Color(self.primary_color)),
            self.theme
                .view()
                .map(|message| ConfigMessage::Theme(message)),
        ]
        .spacing(10)
        .into();

        let component: Element<_> =
            scrollable(column![paths, theme].padding([20, 50]).spacing(20))
                .into();

        component
    }
}
