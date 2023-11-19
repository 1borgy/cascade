use std::{
    backtrace::Backtrace,
    default::Default,
    fmt::{Debug, Display},
};

use cascade::config::{CascadeConfig, ConfigError};
use enum_iterator::{all, Sequence};
use iced::{
    alignment, font,
    widget::{column, container, row, text, Button},
    window, Application, Command, Element, Length,
};
use thiserror::Error;

use crate::{
    about, resources,
    theming::config_to_iced_theme,
    views::{
        config::{ConfigMessage, ConfigView},
        dashboard::{DashboardMessage, DashboardView},
        View,
    },
};

#[derive(Error, Debug)]
pub enum CascadeError {
    #[error("an error occurred when loading/writing config")]
    ConfigError {
        #[from]
        source: ConfigError,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone)]
pub enum CascadeMessage {
    // cannot put Result<window::icon::Icon, window::icon::Error> in this variant
    // because they do not implement Clone
    WindowIconLoaded(bool),
    IconFontLoaded(Result<(), font::Error>),

    ComponentChanged(ViewType),

    // View messages
    Dashboard(DashboardMessage),
    Config(ConfigMessage),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Sequence)]
pub enum ViewType {
    #[default]
    Dashboard,
    Config,
}

impl Display for ViewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ViewType::Dashboard => "dashboard",
                ViewType::Config => "config",
            }
        )
    }
}

struct ViewManager {
    current_view: ViewType,

    dashboard: DashboardView,
    config: ConfigView,
}

impl ViewManager {
    fn new(config: &CascadeConfig) -> Self {
        ViewManager {
            current_view: Default::default(),

            dashboard: DashboardView::new(config.clone()),
            config: ConfigView::new(config.clone()),
        }
    }

    fn change_component(&mut self, view: ViewType) {
        self.current_view = view;
    }

    fn view<'a>(&'a self) -> Element<CascadeMessage> {
        match self.current_view {
            ViewType::Dashboard => {
                self.dashboard.view().map(CascadeMessage::Dashboard).into()
            }

            ViewType::Config => {
                self.config.view().map(CascadeMessage::Config).into()
            }
        }
    }
}

pub struct Cascade {
    view_manager: ViewManager,
    config: CascadeConfig,
    theme: iced::Theme,
}

impl Cascade {
    fn settings() -> iced::Settings<()> {
        iced::Settings {
            window: window::Settings {
                size: (720, 520),
                min_size: Some((720, 520)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn start() -> iced::Result {
        Self::run(Self::settings())
    }

    fn set_config(&mut self, config: CascadeConfig) {
        // TODO: code reuse
        self.theme = config_to_iced_theme(&config.theme);
        // TODO: lmfao what is even the point of the view manager
        self.view_manager.dashboard.set_config(config.clone());
        self.view_manager.config.set_config(config.clone());
        self.config = config;

        if let Err(err) = self.config.write() {
            log::error!("could not write config: {err}")
        }
    }
}

impl Application for Cascade {
    type Message = CascadeMessage;
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn title(&self) -> String {
        format!("cascade v{}", about::VERSION)
    }

    fn new(_flags: ()) -> (Cascade, Command<CascadeMessage>) {
        let config = CascadeConfig::load_or_create().unwrap_or_default();
        let theme = config_to_iced_theme(&config.theme);
        let view_manager = ViewManager::new(&config);

        (
            Cascade {
                view_manager,
                config,
                theme,
            },
            //
            Command::batch(vec![
                // load icon font
                font::load(iced_aw::graphics::icons::ICON_FONT_BYTES)
                    .map(CascadeMessage::IconFontLoaded),
                // load window icon
                window::icon::from_file_data(
                    resources::WINDOW_ICON_BYTES,
                    None,
                )
                .map(|icon| {
                    window::change_icon(icon)
                        .map(CascadeMessage::WindowIconLoaded)
                })
                // fucky workaround since window::icon::Icon and window::icon::Error
                // don't implement Clone
                .unwrap_or_else(|err| {
                    log::error!("could not load window icon: {}", err);
                    Command::none()
                }),
            ]),
        )
    }

    fn update(
        &mut self,
        event: CascadeMessage,
    ) -> iced::Command<CascadeMessage> {
        match event {
            CascadeMessage::IconFontLoaded(Ok(_)) => {
                log::info!("icon font loaded");
            }
            CascadeMessage::IconFontLoaded(Err(err)) => {
                log::error!("could not load icon font: {:?}", err);
            }
            CascadeMessage::WindowIconLoaded(false) => (),
            CascadeMessage::WindowIconLoaded(true) => {
                log::info!("window icon loaded");
            }
            CascadeMessage::ComponentChanged(component) => {
                self.view_manager.change_component(component);
            }
            CascadeMessage::Dashboard(inner_message) => {
                self.view_manager.dashboard.update(inner_message);
            }

            CascadeMessage::Config(inner_message) => {
                self.view_manager
                    .config
                    .update(inner_message)
                    .map(|message| match message {
                        ConfigMessage::ConfigChanged(config) => {
                            self.set_config(config)
                        }
                        _ => (),
                    });
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<CascadeMessage> {
        let switcher: Element<_> = column![row(all::<ViewType>()
            .map(|component_type| {
                button(format!("{}", component_type).as_str())
                    .on_press(CascadeMessage::ComponentChanged(component_type))
                    .width(Length::Fill)
                    .into()
            })
            .collect())
        .width(Length::Fill)
        .spacing(2),]
        .align_items(alignment::Alignment::Center)
        .into();

        let current_component = self.view_manager.view();

        let fill_container: Element<_> = container(current_component)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let stack: Element<_> = column![switcher, fill_container].into();

        stack //.explain(iced::Color::WHITE)
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

fn button<'a, Message: Clone>(label: &str) -> Button<'a, Message> {
    iced::widget::button(
        text(label).horizontal_alignment(alignment::Horizontal::Center),
    )
    .padding(10)
    .width(100)
}
