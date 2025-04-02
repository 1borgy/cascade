use std::path::PathBuf;

use iced::{
    event,
    keyboard::{self, key},
    widget::container,
    Event, Length, Padding, Subscription, Task,
};

use crate::{
    config::{Config, Format, Selections},
    dashboard, paths, tasks, Element, Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
    Dashboard(dashboard::Message),

    WroteConfig(Result<usize, tasks::Error>),
    WroteSelections(Result<usize, tasks::Error>),

    EventOccurred(Event),
}

pub struct Cascade {
    config_path: PathBuf,
    selections_path: PathBuf,

    config: Config,
    debug: bool,
    theme: Theme,

    dashboard: dashboard::Dashboard,
}

impl Cascade {
    pub fn new(
        flags: (PathBuf, Config, Selections, Theme, bool),
    ) -> (Self, Task<Message>) {
        let (cascade_dir, config, selections, theme, debug) = flags;
        let backup_dir = paths::backup_dir(&cascade_dir);

        let (dashboard, dashboard_command) = dashboard::Dashboard::new(
            config.source_path.clone(),
            config.saves_dir.clone(),
            backup_dir,
            config.default_selection,
            selections,
            config.trickset,
            config.scales,
        );

        let config_path = paths::config(&cascade_dir);
        let selections_path = paths::selections(&cascade_dir);

        (
            Cascade {
                config_path,
                selections_path,
                theme,
                config,
                debug,
                dashboard,
            },
            dashboard_command.map(Message::Dashboard),
        )
    }

    fn write_config(&self) -> Task<Message> {
        Task::perform(
            tasks::write(
                self.config.clone(),
                self.config_path.clone(),
                Format::Toml,
            ),
            Message::WroteConfig,
        )
    }

    fn write_selections(&self, selections: Selections) -> Task<Message> {
        Task::perform(
            tasks::write(
                selections.clone(),
                self.selections_path.clone(),
                Format::Ron,
            ),
            Message::WroteSelections,
        )
    }

    pub fn scale_factor(&self) -> f64 {
        self.config.scale_factor
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dashboard(message) => {
                let (command, event) = self.dashboard.update(message);

                Task::batch(vec![
                    command.map(Message::Dashboard),
                    match event {
                        Some(dashboard::Event::SetSavesDir(saves_dir)) => {
                            self.config.saves_dir = Some(saves_dir.clone());

                            Task::batch(vec![
                                self.dashboard
                                    .set_saves_dir(saves_dir)
                                    .map(Message::Dashboard),
                                self.write_config(),
                            ])
                        }
                        Some(dashboard::Event::SetSelections(selections)) => {
                            self.write_selections(selections)
                        }
                        Some(dashboard::Event::SetDefaultSelection(
                            default_selection,
                            selections,
                        )) => {
                            self.config.default_selection = default_selection;

                            Task::batch(vec![
                                self.write_config(),
                                self.write_selections(selections),
                            ])
                        }

                        Some(dashboard::Event::SetSourcePath(path)) => {
                            self.config.source_path = Some(path);
                            self.write_config()
                        }

                        Some(dashboard::Event::SetTrickset(value)) => {
                            self.config.trickset = value;
                            self.write_config()
                        }

                        Some(dashboard::Event::SetScales(value)) => {
                            self.config.scales = value;
                            self.write_config()
                        }

                        None => Task::none(),
                    },
                ])
            }
            Message::WroteConfig(Ok(_)) | Message::WroteSelections(Ok(_)) => {
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
            Message::EventOccurred(event) => match event {
                Event::Keyboard(event) => match event {
                    keyboard::Event::KeyPressed {
                        physical_key: key::Physical::Code(key::Code::Equal),
                        modifiers: keyboard::Modifiers::CTRL,
                        ..
                    } => {
                        if self.config.scale_factor < 5. {
                            self.config.scale_factor += 0.1;
                            self.write_config()
                        } else {
                            Task::none()
                        }
                    }
                    keyboard::Event::KeyPressed {
                        physical_key: key::Physical::Code(key::Code::Minus),
                        modifiers: keyboard::Modifiers::CTRL,
                        ..
                    } => {
                        if self.config.scale_factor > 0.2 {
                            self.config.scale_factor -= 0.1;
                            self.write_config()
                        } else {
                            Task::none()
                        }
                    }
                    _ => Task::none(),
                },
                _ => Task::none(),
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> =
            container(self.dashboard.view().map(Message::Dashboard))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::new(20.))
                .into();

        match self.debug {
            true => content.explain(iced::Color::WHITE),
            false => content,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}
