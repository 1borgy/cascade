use std::path::PathBuf;

use iced::{
    event,
    keyboard::{self, key},
    widget::{container, pick_list},
    Event, Length, Padding, Subscription, Task,
};
use serde::Serialize;

use crate::{
    paths::Paths,
    state::{Game, State},
    tasks, Element, Result, Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
    GameSelected(Game),
    EventOccurred(Event),

    WroteFile(Result<usize>),
}

pub struct Cascade {
    paths: Paths,
    theme: Theme,
    state: State,
}

impl Cascade {
    pub fn new(paths: Paths, theme: Theme, state: State) -> (Self, Task<Message>) {
        (
            Cascade {
                paths,
                theme,
                state,
            },
            Task::none(),
        )
    }

    fn write_state(&self, obj: impl Serialize + Send + 'static, to: PathBuf) -> Task<Message> {
        Task::perform(tasks::write_ron(obj, to), Message::WroteFile)
    }

    fn write_app_state(&self) -> Task<Message> {
        self.write_state(self.state.app.clone(), self.paths.app.clone())
    }

    fn write_game_state(&self) -> Task<Message> {
        match self.state.app.game {
            Game::Thug2 => {
                self.write_state(self.state.game.thug2.clone(), self.paths.thug2.clone())
            }
            Game::Thaw => self.write_state(self.state.game.thaw.clone(), self.paths.thaw.clone()),
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.state.app.scale_factor
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GameSelected(game) => {
                self.state.app.game = game;
                Task::none()
            }
            // Ctrl+=
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                physical_key: key::Physical::Code(key::Code::Equal),
                modifiers: keyboard::Modifiers::CTRL,
                ..
            })) => {
                if self.state.app.scale_factor < 5. {
                    self.state.app.scale_factor += 0.1;
                    self.write_app_state()
                } else {
                    Task::none()
                }
            }
            // Ctrl+-
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                physical_key: key::Physical::Code(key::Code::Minus),
                modifiers: keyboard::Modifiers::CTRL,
                ..
            })) => {
                if self.state.app.scale_factor > 0.2 {
                    self.state.app.scale_factor -= 0.1;
                    self.write_app_state()
                } else {
                    Task::none()
                }
            }
            Message::EventOccurred(_) => Task::none(),
            Message::WroteFile(Ok(_)) => Task::none(),
            Message::WroteFile(Err(err)) => {
                log::error!("error writing file: {}", err);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<Message> = container(pick_list(
            Game::ALL,
            Some(self.state.app.game),
            Message::GameSelected,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(20.))
        .into();

        content
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}
