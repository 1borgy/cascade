use std::{
    collections::hash_map,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    Event, Font, Length, Padding, Subscription, Task,
    alignment::Vertical,
    event,
    font::Weight,
    keyboard::{self, key},
    widget::{Column, Row, button, checkbox, container, pick_list, scrollable, text, tooltip},
};
use indexmap::IndexMap;
use rfd::AsyncFileDialog;
use serde::Serialize;

use crate::{
    Element, Error, Result, Theme, fonts,
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

    Run,
    EntryProcessed(String, Result<()>),

    WroteFile(Result<usize>),
}

type LoadEntries = Box<dyn Fn(&PathBuf) -> Vec<cascade_core::Entry>>;

struct Context {
    state_path: PathBuf,
    state: GameState,
    load_entries: LoadEntries,
    entries: Vec<cascade_core::Entry>,
    filter_name: &'static str,
    filter_extension: &'static str,
}

impl Context {
    pub fn new(
        state_path: PathBuf,
        state: GameState,
        load_entries: LoadEntries,
        filter_name: &'static str,
        filter_extension: &'static str,
    ) -> Self {
        let mut slf = Self {
            state_path,
            state,
            load_entries,
            entries: Vec::new(),
            filter_name,
            filter_extension,
        };
        slf.refresh();
        slf
    }

    pub fn refresh(&mut self) {
        if let Some(to_dir) = &self.state.to_dir {
            let entries = (self.load_entries)(to_dir);
            if entries.len() > 0 {
                self.entries = entries;
            }
        }
    }

    pub fn write(&mut self) -> Task<Message> {
        Task::perform(
            tasks::write_ron(self.state.clone(), self.state_path.clone()),
            Message::WroteFile,
        )
    }
}

struct Contexts {
    thps4: Context,
    thug2: Context,
    thaw: Context,
}

impl Contexts {
    pub fn new(state: &State, paths: &Paths) -> Contexts {
        Self {
            thps4: Context::new(
                paths.thps4.clone(),
                state.game.thps4.clone(),
                Box::new(cascade_thps4::find_entries),
                cascade_thps4::FILTER_NAME,
                cascade_thps4::FILTER_EXTENSION,
            ),
            thug2: Context::new(
                paths.thug2.clone(),
                state.game.thug2.clone(),
                Box::new(cascade_thug2::find_entries),
                cascade_thug2::FILTER_NAME,
                cascade_thug2::FILTER_EXTENSION,
            ),
            thaw: Context::new(
                paths.thaw.clone(),
                state.game.thaw.clone(),
                Box::new(cascade_thaw::find_entries),
                cascade_thaw::FILTER_NAME,
                cascade_thaw::FILTER_EXTENSION,
            ),
        }
    }
}

#[derive(Debug, Clone)]
enum Status {
    // i.e. last known status of processing an entry
    InProgress,
    Success,
    Error(Error),
}

pub struct Cascade {
    paths: Paths,
    theme: Theme,
    state: State,
    contexts: Contexts,

    queue: IndexMap<String, Status>,

    enabled: bool,
}

impl Cascade {
    pub fn new(paths: Paths, theme: Theme, state: State) -> (Self, Task<Message>) {
        let contexts = Contexts::new(&state, &paths);
        let queue = IndexMap::new();
        (
            Cascade {
                paths,
                theme,
                state,
                contexts,
                queue,
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

    fn context(&self) -> &Context {
        match self.state.app.game {
            Game::Thps4 => &self.contexts.thps4,
            Game::Thug2 => &self.contexts.thug2,
            Game::Thaw => &self.contexts.thaw,
        }
    }

    fn context_mut(&mut self) -> &mut Context {
        match self.state.app.game {
            Game::Thps4 => &mut self.contexts.thps4,
            Game::Thug2 => &mut self.contexts.thug2,
            Game::Thaw => &mut self.contexts.thaw,
        }
    }

    fn with_context_mut(&mut self, f: impl FnOnce(&mut Context)) -> Task<Message> {
        let context = self.context_mut();
        f(context);
        context.write()
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
            Message::PickFrom => {
                let context = self.context();
                Task::perform(
                    pick_source(context.filter_name, context.filter_extension),
                    Message::SetFrom,
                )
            }
            Message::SetFrom(path) => match path {
                Some(path) => self.with_context_mut(|context| context.state.from = Some(path)),
                None => Task::none(),
            },
            Message::PickToDir => Task::perform(pick_to_dir(), Message::SetToDir),
            Message::SetToDir(dir) => match dir {
                Some(dir) => self.with_context_mut(|context| {
                    context.state.to_dir = Some(dir);
                    context.refresh()
                }),
                None => Task::none(),
            },
            Message::SetTricksetFlag(selected) => {
                self.with_context_mut(|context| context.state.trickset = selected)
            }
            Message::SetScalesFlag(selected) => {
                self.with_context_mut(|context| context.state.scales = selected)
            }
            Message::GameSelected(game) => {
                self.state.app.game = game;
                self.write_app_state()
            }
            Message::ToggleDefaultSelection => self.with_context_mut(|context| {
                context.state.default_selection = !context.state.default_selection;
                context.state.selections.clear();
            }),
            Message::ToggleSelection(name) => self.with_context_mut(|context| {
                match context.state.selections.entry(name) {
                    // If occupied, remove to use default selection
                    hash_map::Entry::Occupied(entry) => {
                        entry.remove();
                    }
                    // If vacant, insert since no longer using default selection
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(!context.state.default_selection);
                    }
                }
            }),
            Message::Run => self.run().unwrap_or_else(|err| {
                log::error!("runtime error: {}", err);
                Task::none()
            }),
            Message::EntryProcessed(name, result) => {
                let new_status = match result {
                    Ok(_) => Status::Success,
                    Err(err) => {
                        log::error!("error for entry {}: {:?}", name, err);
                        Status::Error(err)
                    }
                };

                self.queue.entry(name).and_modify(|status| {
                    *status = new_status;
                });

                if self.queue.values().all(|entry| match entry {
                    Status::InProgress => false,
                    Status::Success | Status::Error(_) => true,
                }) {
                    self.enabled = true;
                }

                Task::none()
            }
        }
    }

    fn selected_entries<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = &'a cascade_core::Entry> {
        context.entries.iter().filter(|entry| {
            *context
                .state
                .selections
                .get(entry.name())
                .unwrap_or(&context.state.default_selection)
        })
    }

    fn run(&mut self) -> Result<Task<Message>> {
        let datetime = time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc());

        let backup_dir = self.paths.backup.join(format!(
            "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
            datetime.year(),
            u8::from(datetime.month()),
            datetime.day(),
            datetime.hour(),
            datetime.minute(),
            datetime.second()
        ));

        // TODO: async?
        log::info!("backing up to {:?}", backup_dir);
        std::fs::create_dir_all(&backup_dir)?;

        self.queue = self
            .selected_entries(self.context())
            .map(|entry| (entry.name().clone(), Status::InProgress))
            .collect::<IndexMap<_, _>>();

        self.enabled = false;
        let context = self.context();
        match &context.state.from {
            Some(from) => {
                let from_entry = cascade_core::Entry::create(from)?;
                let flags = cascade_core::Flags {
                    summary: false,
                    trickset: context.state.trickset,
                    scales: context.state.scales,
                };

                match self.state.app.game {
                    Game::Thps4 => run::<cascade_thps4::Save, cascade_thps4::Cas>(
                        from_entry,
                        context.entries.iter().cloned(),
                        &backup_dir,
                        flags,
                    ),
                    Game::Thug2 => run::<cascade_thug2::Save, cascade_thug2::Cas>(
                        from_entry,
                        context.entries.iter().cloned(),
                        &backup_dir,
                        flags,
                    ),
                    Game::Thaw => run::<cascade_thaw::Save, cascade_thaw::Cas>(
                        from_entry,
                        context.entries.iter().cloned(),
                        &backup_dir,
                        flags,
                    ),
                }
            }
            None => {
                self.enabled = true;
                log::info!("cannot run with no 'from' entry");
                Ok(Task::none())
            }
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

    fn view_left(&self, context: &Context) -> Element<'_, Message> {
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
                    .push(self.view_path(&context.state.from)),
            )
            .push(
                checkbox("trickset", context.state.trickset)
                    .on_toggle_maybe(self.enabled.then_some(Message::SetTricksetFlag)),
            )
            .push(
                checkbox("scales", context.state.scales)
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

    fn view_center(&self, context: &Context) -> Element<'_, Message> {
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
                    .push(self.view_path(&context.state.to_dir)),
            )
            .push(
                button(
                    text(match context.state.default_selection {
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
            .push(scrollable(self.view_entries()))
            .into()
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
                            .on_press_maybe(self.enabled.then_some(Message::Run)),
                    )
                    .push(heading("queue")),
            )
            // TODO:
            .push(self.view_queue())
            .width(Length::Fill)
            .into()
    }

    fn view_entries(&self) -> Element<'_, Message> {
        let context: &Context = self.context();
        context
            .entries
            .iter()
            .fold(Column::new().spacing(2), |column, entry| {
                let name = entry.name();
                let selected = context
                    .state
                    .selections
                    .get(name)
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
                    .padding(5)
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
                    .padding(5)
                    .align_y(Vertical::Center),
                tooltip::Position::Bottom,
            )
            .gap(10)
            .style(theme::container::bordered)
            .into()
        }
    }

    fn view_queue(&self) -> Element<'_, Message> {
        let context = self.context();
        scrollable(self.selected_entries(context).fold(
            Column::new().spacing(2),
            |column, entry| {
                let name = entry.name();
                let status = self.queue.get(name);
                let style = match status {
                    Some(Status::InProgress) => theme::button::entry_warning,
                    Some(Status::Success) => theme::button::entry_success,
                    Some(Status::Error(_)) => theme::button::entry_danger,
                    None => theme::button::entry_queued,
                };
                let button = button(text(name))
                    .style(style)
                    .on_press_maybe(
                        self.enabled
                            .then_some(Message::ToggleSelection(name.clone())),
                    )
                    .width(Length::Fill);

                let with_tooltip: Element<'_, Message> = match status {
                    Some(Status::InProgress) => tooltip(
                        button,
                        container(text("In progress...")).padding(5),
                        tooltip::Position::Bottom,
                    )
                    .gap(10)
                    .style(theme::container::bordered)
                    .into(),
                    Some(Status::Success) => tooltip(
                        button,
                        container(text("Success!")).padding(5),
                        tooltip::Position::Bottom,
                    )
                    .gap(10)
                    .style(theme::container::bordered)
                    .into(),
                    Some(Status::Error(err)) => tooltip(
                        button,
                        container(text(format!("{}", err))).padding(5),
                        tooltip::Position::Bottom,
                    )
                    .gap(10)
                    .style(theme::container::bordered)
                    .into(),
                    None => button.into(),
                };

                column.push(with_tooltip)
            },
        ))
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}

async fn pick_source(name: &str, extension: &str) -> Option<PathBuf> {
    Some(
        AsyncFileDialog::new()
            .add_filter(name, &[extension])
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

fn run<Save, Cas>(
    from: cascade_core::Entry,
    to: impl IntoIterator<Item = cascade_core::Entry>,
    backup_dir: &PathBuf,
    flags: cascade_core::Flags,
) -> Result<Task<Message>>
where
    Save: cascade_core::Save + 'static,
    Cas: cascade_core::Cas<Save = Save> + 'static,
{
    let transform = Arc::new(Cas::parse(&Save::read(&mut from.reader()?)?)?.mask(flags));

    Ok(Task::batch(to.into_iter().map(|entry| {
        Task::perform(
            process_entry(entry.clone(), backup_dir.clone(), Arc::clone(&transform)),
            move |result| Message::EntryProcessed(entry.name().clone(), result),
        )
    })))
}

async fn process_entry<Save, Cas>(
    entry: cascade_core::Entry,
    backup_dir: impl AsRef<Path>,
    transform: Arc<Cas>,
) -> Result<()>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let backup_dir = backup_dir.as_ref();

    let backup_entry = entry.with_dir(backup_dir);
    let backup_path = backup_entry.path();

    let path = entry.path();

    log::info!("backing up {:?} to {:?}", path, backup_path);
    tokio::fs::copy(&path, &backup_path).await?;

    let mut save = Save::read(&mut entry.reader()?)?;

    transform.modify(&mut save)?;
    save.write(&mut entry.writer()?)?;
    entry.rewrite_metadata()?;

    log::info!("overwrote save at {:?}", path);

    Ok(())
}
