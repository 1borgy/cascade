use std::{default::Default, fmt::Debug, path::PathBuf};

use enum_iterator::{all, Sequence};
use iced::{
    font,
    widget::{self, button, text, Column, Row},
    Element, Length, Padding, Task,
};

use crate::{
    config::{Config, Format, Selections},
    fonts, paths,
    screen::{dashboard, options},
    tasks,
};

#[derive(Debug, Clone)]
pub enum Message {
    Dashboard(dashboard::Message),
    Options(options::Message),

    SetScreen(Screen),

    LoadedFont(Result<(), font::Error>),
    WroteConfig(Result<usize, tasks::Error>),
    WroteSelections(Result<usize, tasks::Error>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Sequence)]
pub enum Screen {
    #[default]
    Dashboard,
    Options,
}

impl Screen {
    pub fn title(&self) -> &str {
        match self {
            Screen::Dashboard => "dashboard",
            Screen::Options => "options",
        }
    }
}

pub struct Cascade {
    config_path: PathBuf,
    selections_path: PathBuf,

    config: Config,
    debug: bool,

    selected_view: Screen,

    dashboard: dashboard::Dashboard,
    options: options::Options,
}

impl Cascade {
    fn tab_bar(&self) -> Element<Message> {
        all::<Screen>()
            .fold(Row::new(), |row, screen| {
                let is_selected = self.selected_view == screen;
                row.push(
                    button(
                        text(screen.title().to_string())
                            .font(fonts::IOSEVKA_BOLD),
                    )
                    .on_press(Message::SetScreen(screen))
                    .style(match is_selected {
                        true => widget::button::success,
                        false => widget::button::secondary,
                    })
                    .padding(Padding {
                        top: 5.,
                        bottom: 5.,
                        left: 10.,
                        right: 10.,
                    }),
                )
            })
            .spacing(10)
            .into()
    }

    fn write_config(&self) -> Task<Message> {
        Task::perform(
            tasks::write_serializable(
                self.config.clone(),
                self.config_path.clone(),
                Format::Ron,
            ),
            Message::WroteConfig,
        )
    }

    fn write_selections(&self, selections: Selections) -> Task<Message> {
        Task::perform(
            tasks::write_serializable(
                selections.clone(),
                self.selections_path.clone(),
                Format::Ron,
            ),
            Message::WroteSelections,
        )
    }

    pub fn new(
        flags: (PathBuf, Config, Selections, bool),
    ) -> (Self, Task<Message>) {
        let (cascade_dir, config, selections, debug) = flags;

        let (dashboard, dashboard_command) = dashboard::Dashboard::new(
            &cascade_dir,
            config.saves_dir.clone(),
            config.default_selection,
            selections,
        );

        let (options, options_command) = options::Options::new(
            config.saves_dir.clone(),
            config.theme.clone(),
            config.scale_factor,
        );

        let selected_view = Screen::default();

        let config_path = paths::config(&cascade_dir);
        let selections_path = paths::selections(&cascade_dir);

        (
            Cascade {
                config_path,
                selections_path,
                config,
                debug,
                selected_view,
                dashboard,
                options,
            },
            Task::batch(vec![
                dashboard_command.map(Message::Dashboard),
                options_command.map(Message::Options),
                font::load(fonts::IOSEVKA_REGULAR_BYTES)
                    .map(Message::LoadedFont),
                font::load(fonts::IOSEVKA_BOLD_BYTES).map(Message::LoadedFont),
            ]),
        )
    }

    pub fn scale_factor(&self) -> f64 {
        self.config.scale_factor
    }

    pub fn theme(&self) -> iced::Theme {
        self.config.theme.into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Options(message) => {
                let (command, event) = self.options.update(message);

                Task::batch(vec![
                    command.map(Message::Options),
                    match event {
                        Some(options::Event::SavesDirChanged(saves_dir)) => {
                            self.config.saves_dir = Some(saves_dir.clone());

                            Task::batch(vec![
                                self.dashboard
                                    .set_saves_dir(saves_dir)
                                    .map(Message::Dashboard),
                                self.write_config(),
                            ])
                        }

                        Some(options::Event::ScaleFactorChanged(
                            scale_factor,
                        )) => {
                            self.config.scale_factor = scale_factor;
                            self.write_config()
                        }

                        Some(options::Event::ThemeChanged(theme)) => {
                            self.config.theme = theme;
                            self.write_config()
                        }

                        None => Task::none(),
                    },
                ])
            }

            Message::Dashboard(message) => {
                let (command, event) = self.dashboard.update(message);

                Task::batch(vec![
                    command.map(Message::Dashboard),
                    match event {
                        Some(dashboard::Event::SelectionsUpdated(
                            selections,
                        )) => self.write_selections(selections),

                        None => Task::none(),
                    },
                ])
            }

            Message::SetScreen(screen) => {
                log::info!("select screen: {}", screen.title());
                self.selected_view = screen;
                Task::none()
            }

            Message::LoadedFont(Ok(_))
            | Message::WroteConfig(Ok(_))
            | Message::WroteSelections(Ok(_)) => Task::none(),

            Message::LoadedFont(Err(e)) => {
                log::info!("error loading icon font: {:?}", e);
                Task::none()
            }
            Message::WroteConfig(Err(err)) => {
                log::info!("error writing config: {:?}", err);
                Task::none()
            }
            Message::WroteSelections(Err(err)) => {
                log::info!("error writing selections: {:?}", err);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let screen = match self.selected_view {
            Screen::Dashboard => self.dashboard.view().map(Message::Dashboard),
            Screen::Options => self.options.view().map(Message::Options),
        };

        let tab_bar = self.tab_bar();

        let content: Element<Message> = Column::new()
            .push(tab_bar)
            .push(screen)
            .spacing(10.)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(20.))
            .into();

        match self.debug {
            true => content.explain(iced::Color::WHITE),
            false => content,
        }
    }
}
