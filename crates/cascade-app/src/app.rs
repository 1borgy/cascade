use std::path::PathBuf;

use cascade_core::Entry;
use iced::{
    Event, Font, Length, Padding, Subscription, Task,
    alignment::Vertical,
    event,
    font::Weight,
    keyboard::{self, key},
    widget::{Column, Row, button, checkbox, container, pick_list, text, tooltip},
};
use rfd::AsyncFileDialog;
use serde::Serialize;

use crate::{
    Element, Result, Theme, fonts,
    paths::Paths,
    state::{Game, GameState, State},
    tasks, theme,
    widget::{self, heading},
};

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),

    SetTricksetFlag(bool),
    SetScalesFlag(bool),
    GameSelected(Game),

    PickFrom,
    SetFrom(Option<PathBuf>),

    PickToDir,
    SetToDir(Option<PathBuf>),

    ToggleDefaultSelection,
    ToggleSelection(String),

    Start,

    WroteFile(Result<usize>),
}

struct Context<Core: cascade_core::Core> {
    state_path: PathBuf,
    state: GameState,
    core: Core,
    entries: Vec<Core::Entry>,
}

impl<Core: cascade_core::Core> Context<Core> {
    pub fn new(state_path: PathBuf, state: GameState, core: Core) -> Self {
        let mut slf = Self {
            state_path,
            state,
            core,
            entries: Vec::new(),
        };
        slf.refresh();
        slf
    }

    pub fn refresh(&mut self) {
        if let Some(to_dir) = &self.state.to_dir {
            match self.core.entries(to_dir) {
                Ok(entries) => self.entries = entries.collect(),
                Err(err) => log::warn!("error refreshing entries: {}", err),
            };
        }
    }

    pub fn write(&mut self) -> Task<Message> {
        Task::perform(
            tasks::write_ron(self.state.clone(), self.state_path.clone()),
            Message::WroteFile,
        )
    }
}

impl Contexts {
    pub fn new(state: &State, paths: &Paths) -> Contexts {
        Self {
            thug2: Context::new(
                paths.thug2.clone(),
                state.game.thug2.clone(),
                cascade_thug2::Core {},
            ),
            thaw: Context::new(
                paths.thaw.clone(),
                state.game.thaw.clone(),
                cascade_thaw::Core {},
            ),
        }
    }
}

pub struct Cascade {
    paths: Paths,
    theme: Theme,
    state: State,
    contexts: Contexts,

    enabled: bool,
}

impl Cascade {
    pub fn new(paths: Paths, theme: Theme, state: State) -> (Self, Task<Message>) {
        let contexts = Contexts::new(&state, &paths);
        (
            Cascade {
                paths,
                theme,
                state,
                contexts,
                enabled: true,
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

    fn context<Core: cascade_core::Core>(&self) -> &Context<Core> {
        match self.state.app.game {
            Game::Thug2 => &self.contexts.thug2,
            Game::Thaw => &self.contexts.thaw,
        }
    }

    fn with_context_mut(&mut self, f: impl FnOnce(&mut GameState), refresh: bool) -> Task<Message> {
        match self.state.app.game {
            Game::Thug2 => {
                f(&mut self.contexts.thug2.state);
                if refresh {
                    self.contexts.thug2.refresh();
                }
                self.contexts.thug2.write()
            }
            Game::Thaw => {
                f(&mut self.contexts.thaw.state);
                if refresh {
                    self.contexts.thaw.refresh();
                }
                self.contexts.thaw.write()
            }
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
            Message::PickFrom => Task::perform(pick_source(), Message::SetFrom),
            Message::SetFrom(path) => match path {
                Some(path) => self.with_context_mut(|s| s.from = Some(path), false),
                None => Task::none(),
            },
            Message::PickToDir => Task::perform(pick_to_dir(), Message::SetToDir),
            Message::SetToDir(dir) => match dir {
                Some(dir) => self.with_context_mut(|s| s.to_dir = Some(dir), true),
                None => Task::none(),
            },
            Message::SetTricksetFlag(selected) => {
                self.with_context_mut(|s| s.trickset = selected, false)
            }
            Message::SetScalesFlag(selected) => {
                self.with_context_mut(|s| s.scales = selected, false)
            }
            Message::GameSelected(game) => {
                self.state.app.game = game;
                self.write_app_state()
            }
            Message::ToggleDefaultSelection => {
                self.with_context_mut(|s| s.default_selection = !s.default_selection, false)
            }
            Message::Start => Task::none(),
            Message::ToggleSelection(_) => todo!(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let current_game = self.context();
        Row::new()
            .push(self.view_left(current_game))
            .push(self.view_center(current_game))
            .push(self.view_right())
            .width(Length::Fill)
            .spacing(10)
            .padding(Padding::new(20.))
            .into()
    }

    pub fn view_left(&self, game_state: &GameState) -> Element<'_, Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .height(Length::Shrink)
                    .push(
                        button(text("\u{E802}").font(fonts::ICONS_FONT))
                            .on_press_maybe(self.enabled.then_some(Message::PickFrom)),
                    )
                    .push(heading("from"))
                    .push(self.view_path(&game_state.from)),
            )
            .push(
                checkbox("trickset", game_state.trickset)
                    .on_toggle_maybe(self.enabled.then_some(Message::SetTricksetFlag)),
            )
            .push(
                checkbox("scales", game_state.scales)
                    .on_toggle_maybe(self.enabled.then_some(Message::SetScalesFlag)),
            )
            .push(pick_list(
                Game::ALL,
                Some(self.state.app.game),
                Message::GameSelected,
            ))
            .push(Row::new().height(Length::Fill).align_y(Vertical::Bottom))
            .width(Length::Fill)
            .into()
    }

    fn view_center(&self, game_state: &GameState) -> Element<'_, Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .height(Length::Shrink)
                    .push(
                        button(text("\u{E802}").font(fonts::ICONS_FONT))
                            .on_press_maybe(self.enabled.then_some(Message::PickToDir)),
                    )
                    .push(heading("to"))
                    .push(self.view_path(&game_state.to_dir)),
            )
            .push(
                button(
                    text(match game_state.default_selection {
                        true => "deselect all",
                        false => "select all",
                    })
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    }),
                )
                .style(theme::button::secondary)
                .on_press_maybe(self.enabled.then_some(Message::ToggleDefaultSelection))
                .width(Length::Fill),
            )
            .width(Length::Fill)
            // TODO:
            // .push(scrollable(self.view_entries()))
            .into()
    }

    fn view_entries<Core: cascade_core::Core>(&self) -> Element<Message> {
        let context: &Context<Core> = self.context();
        context
            .entries
            .iter()
            .fold(Column::new().spacing(2), |column, entry| {
                let name = entry.name();
                let selected = context
                    .state
                    .selections
                    .get(&name)
                    .unwrap_or(&context.state.default_selection);

                column.push(widget::entry::selectable(
                    name,
                    *selected,
                    self.enabled
                        .then_some(Message::ToggleSelection(entry.name().clone())),
                ))
            })
            .into()
    }

    fn view_path(&self, path: &Option<PathBuf>) -> Element<'_, Message> {
        if let Some(path) = &path {
            let filepath = format!("{}", path.display());
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("(invalid path)")
                .to_string();

            tooltip(
                container(text(filename))
                    .style(theme::container::monobox)
                    .padding(10)
                    .align_y(Vertical::Center),
                container(text(filepath))
                    .padding(10)
                    .align_y(Vertical::Center),
                tooltip::Position::Bottom,
            )
            .gap(10)
            .style(theme::container::bordered)
            .into()
        } else {
            tooltip(
                container(text("(none)"))
                    .style(theme::container::monobox)
                    .padding(10)
                    .align_y(Vertical::Center),
                container(text("please select a path"))
                    .padding(10)
                    .align_y(Vertical::Center),
                tooltip::Position::Bottom,
            )
            .gap(10)
            .style(theme::container::bordered)
            .into()
        }
    }

    fn view_right(&self) -> Element<'_, Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .push(
                        button(text("\u{E803}").font(fonts::ICONS_FONT))
                            .on_press_maybe(self.enabled.then_some(Message::Start)),
                    )
                    .push(heading("queue")),
            )
            // TODO:
            // .push(self.view_queue())
            .width(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}

async fn pick_source() -> Option<PathBuf> {
    // TODO: Change to -Progress for THAW
    Some(
        AsyncFileDialog::new()
            .add_filter("CAS file (.SKA)", &["SKA"])
            .add_filter("Any file", &[""])
            .pick_file()
            .await?
            .path()
            .into(),
    )
}

async fn pick_to_dir() -> Option<PathBuf> {
    Some(AsyncFileDialog::new().pick_folder().await?.path().into())
}

struct Contexts {
    thug2: Context<cascade_thug2::Core>,
    thaw: Context<cascade_thaw::Core>,
}
