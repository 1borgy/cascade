use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    result,
    sync::Arc,
};

use cascade::{
    lut::{self, Lut},
    save::{self, thug_pro},
};
use iced::{
    alignment::Vertical,
    font::Weight,
    widget::{button, checkbox, container, scrollable, text, tooltip},
    Font, Length, Task,
};
use indexmap::IndexMap;
use rfd::AsyncFileDialog;
use tokio::fs;

use crate::{
    config::{Format, Selections},
    fonts, paths, tasks, theme,
    widget::{self, heading},
    Column, Element, Row,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("save error: {0}")]
    Save(#[from] save::Error),

    #[error("paths error: {0}")]
    Path(#[from] paths::Error),

    #[error("LUT error: {0}")]
    Lut(#[from] lut::Error),

    #[error("tasks error: {0}")]
    Tasks(#[from] tasks::Error),

    #[error("thug pro save error: {0}")]
    ThugPro(#[from] thug_pro::Error),

    #[error("error spawning task")]
    Task,

    #[error("no saves dir set")]
    NoSavesDir,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Clone)]
enum Status {
    // i.e. last known status of processing an entry
    InProgress,
    Success,
    #[expect(dead_code)]
    Error(Error),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Components {
    trickset: bool,
    scales: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadedLut(Result<Lut>),
    LoadedCandidates(Result<IndexMap<save::Entry, bool>>),
    LoadedSource(Result<thug_pro::Cas>),

    PickSource,
    SourcePicked(Option<PathBuf>),

    PickSavesDir,
    SavesDirChanged(PathBuf),
    ClosedSavesDirDialog,

    ToggleSelectAll,
    ToggleSelection(save::Entry),
    ToggleTricksetComponent(bool),
    ToggleScalesComponent(bool),

    Start,
    PreProcessDone(Result<(Arc<thug_pro::Cas>, PathBuf)>),
    EntryProcessed(save::Entry, Result<()>),
}

#[derive(Debug, Clone)]
pub enum Event {
    SetSavesDir(PathBuf),
    SetSourcePath(PathBuf),
    SetDefaultSelection(bool, Selections),
    SetSelections(Selections),
    SetTrickset(bool),
    SetScales(bool),
}

pub struct Dashboard {
    backup_dir: PathBuf,
    saves_dir: Option<PathBuf>,
    default_selection: bool,
    enabled: bool,

    source_entry: Option<save::Entry>,
    source: Option<thug_pro::Cas>,

    candidates: IndexMap<save::Entry, bool>,
    components: Components,
    queue: IndexMap<save::Entry, Status>,

    warning_message: Option<String>,

    lut: Option<Lut>,
}

impl Dashboard {
    pub fn new(
        source_path: Option<PathBuf>,
        saves_dir: Option<PathBuf>,
        backup_dir: PathBuf,
        default_selection: bool,
        selections: Selections,
        trickset: bool,
        scales: bool,
    ) -> (Self, Task<Message>) {
        let source_entry = source_path
            .map(|path| save::Entry::at_path(path).ok())
            .flatten();

        let tasks = Task::batch(vec![
            match &source_entry {
                Some(entry) => Task::perform(
                    load_source(entry.clone()),
                    Message::LoadedSource,
                ),
                None => Task::none(),
            },
            Task::perform(
                load_candidates(
                    saves_dir.clone(),
                    selections,
                    default_selection,
                ),
                Message::LoadedCandidates,
            ),
            Task::perform(load_lut(), Message::LoadedLut),
        ]);

        let dashboard = Dashboard {
            enabled: true,
            backup_dir,
            source_entry,
            source: None,
            saves_dir,
            candidates: IndexMap::new(),
            queue: IndexMap::new(),
            default_selection,
            components: Components { scales, trickset },
            warning_message: None,
            lut: None,
        };

        (dashboard, tasks)
    }

    fn selections(&self) -> Selections {
        self.candidates
            .iter()
            .map(|(entry, selected)| (entry.name.clone(), *selected))
            .collect::<HashMap<_, _>>()
            .into()
    }

    pub fn set_saves_dir(
        &mut self,
        saves_dir: impl AsRef<Path>,
    ) -> Task<Message> {
        self.saves_dir = Some(saves_dir.as_ref().into());

        Task::perform(
            load_candidates(
                self.saves_dir.clone(),
                self.selections(),
                self.default_selection,
            ),
            Message::LoadedCandidates,
        )
    }

    fn notify(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        log::warn!("{}", msg);
        self.warning_message = Some(msg);
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::PickSavesDir => (
                Task::perform(pick_saves_dir(), |dir| {
                    dir.map(|dir| Message::SavesDirChanged(dir))
                        .unwrap_or(Message::ClosedSavesDirDialog)
                }),
                None,
            ),
            Message::ClosedSavesDirDialog => (Task::none(), None),
            Message::SavesDirChanged(saves_dir) => {
                self.saves_dir = Some(saves_dir.clone());
                (Task::none(), Some(Event::SetSavesDir(saves_dir.clone())))
            }
            Message::LoadedSource(Ok(content)) => {
                self.source = Some(content);
                (Task::none(), None)
            }
            Message::LoadedSource(Err(err)) => {
                self.notify(format!("error loading source: {}", err));
                (Task::none(), None)
            }

            Message::LoadedCandidates(Ok(entries)) => {
                self.candidates = entries;
                (Task::none(), None)
            }
            Message::LoadedCandidates(Err(err)) => {
                self.notify(format!("error loading selections: {}", err));
                (Task::none(), None)
            }

            Message::LoadedLut(Ok(lut)) => {
                self.lut = Some(lut);
                (Task::none(), None)
            }
            Message::LoadedLut(Err(err)) => {
                self.notify(format!("error loading LUT: {}", err));
                (Task::none(), None)
            }

            Message::PickSource => {
                (Task::perform(pick_source(), Message::SourcePicked), None)
            }

            Message::SourcePicked(Some(path)) => {
                match save::Entry::at_path(path.clone()) {
                    Ok(entry) => {
                        self.source_entry = Some(entry.clone());
                        (
                            Task::perform(
                                load_source(entry),
                                Message::LoadedSource,
                            ),
                            Some(Event::SetSourcePath(path)),
                        )
                    }
                    Err(err) => {
                        self.notify(format!("error picking source: {err}"));
                        (Task::none(), None)
                    }
                }
            }
            Message::SourcePicked(None) => (Task::none(), None),

            Message::ToggleSelectAll => {
                self.default_selection = !self.default_selection;

                for selected in self.candidates.values_mut() {
                    *selected = self.default_selection;
                }

                (
                    Task::none(),
                    Some(Event::SetDefaultSelection(
                        self.default_selection,
                        self.selections(),
                    )),
                )
            }

            Message::ToggleSelection(entry) => {
                if let Some(selected) = self.candidates.get_mut(&entry) {
                    *selected = !*selected;
                }
                (Task::none(), Some(Event::SetSelections(self.selections())))
            }

            Message::ToggleTricksetComponent(selected) => {
                self.components.trickset = selected;
                (Task::none(), Some(Event::SetTrickset(selected)))
            }

            Message::ToggleScalesComponent(selected) => {
                self.components.scales = selected;
                (Task::none(), Some(Event::SetScales(selected)))
            }

            Message::Start => match &self.source {
                Some(source) => {
                    if self.candidates.values().any(|selected| *selected) {
                        self.enabled = false;

                        let datetime = time::OffsetDateTime::now_local()
                            .unwrap_or(time::OffsetDateTime::now_utc());

                        let backup_dir = self.backup_dir.join(format!(
                            "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
                            datetime.year(),
                            u8::from(datetime.month()),
                            datetime.day(),
                            datetime.hour(),
                            datetime.minute(),
                            datetime.second()
                        ));

                        (
                            Task::perform(
                                pre_process(
                                    backup_dir,
                                    source.clone(),
                                    self.components,
                                ),
                                Message::PreProcessDone,
                            ),
                            None,
                        )
                    } else {
                        (Task::none(), None)
                    }
                }
                None => (Task::none(), None),
            },
            Message::PreProcessDone(Ok((transform, backup_dir))) => {
                let selected_entries = self
                    .candidates
                    .iter()
                    .filter_map(|(entry, selected)| {
                        selected.then_some(entry.clone())
                    })
                    .collect::<Vec<_>>();

                self.queue = selected_entries
                    .iter()
                    .map(|entry| (entry.clone(), Status::InProgress))
                    .collect::<IndexMap<_, _>>();

                (
                    Task::batch(
                        selected_entries
                            .iter()
                            .map(|entry| {
                                let entry = entry.clone();
                                Task::perform(
                                    process_entry(
                                        entry.clone(),
                                        backup_dir.clone(),
                                        Arc::clone(&transform),
                                    ),
                                    move |result| {
                                        Message::EntryProcessed(
                                            entry.clone(),
                                            result,
                                        )
                                    },
                                )
                            })
                            .collect::<Vec<_>>(),
                    ),
                    None,
                )
            }
            Message::PreProcessDone(Err(err)) => {
                self.enabled = true;
                self.notify(format!("error during pre-process: {}", err));

                (Task::none(), None)
            }
            Message::EntryProcessed(entry, result) => {
                let new_status = match result {
                    Ok(_) => Status::Success,
                    Err(err) => {
                        self.notify(format!(
                            "error for entry {}: {:?}",
                            entry.name, err
                        ));
                        Status::Error(err)
                    }
                };

                self.queue.entry(entry).and_modify(|status| {
                    *status = new_status;
                });

                if self.queue.values().all(|entry| match entry {
                    Status::InProgress => false,
                    Status::Success | Status::Error(_) => true,
                }) {
                    self.enabled = true;
                }

                (Task::none(), None)
            }
        }
    }

    fn view_source_info(&self) -> Element<Message> {
        if let Some(entry) = &self.source_entry {
            tooltip(
                container(text(entry.filename()))
                    .style(theme::container::monobox)
                    .padding(10)
                    .align_y(Vertical::Center),
                container(text(entry.filepath().display().to_string()))
                    .padding(10)
                    .align_y(Vertical::Center),
                tooltip::Position::Bottom,
            )
            .gap(10)
            .style(theme::container::bordered)
            .into()
        } else {
            container(text("(none)"))
                .style(theme::container::monobox)
                .padding(10)
                .align_y(Vertical::Center)
                .into()
        }
    }

    fn view_saves_dir(&self) -> Element<Message> {
        // :(
        if let Some(dir) = &self.saves_dir {
            if let Some(name) = dir.file_name() {
                if let Some(name) = name.to_str() {
                    return tooltip(
                        container(text(name))
                            .style(theme::container::monobox)
                            .padding(10)
                            .align_y(Vertical::Center),
                        container(text(dir.display().to_string()))
                            .padding(10)
                            .align_y(Vertical::Center),
                        tooltip::Position::Bottom,
                    )
                    .gap(10)
                    .style(theme::container::bordered)
                    .into();
                }
            }
        }
        container(text("(none)"))
            .style(theme::container::monobox)
            .padding(10)
            .align_y(Vertical::Center)
            .into()
    }

    fn view_entries(&self) -> Element<Message> {
        self.candidates
            .iter()
            .fold(Column::new().spacing(2), |column, (entry, selected)| {
                column.push(widget::entry::selectable(
                    &entry,
                    *selected,
                    self.enabled
                        .then_some(Message::ToggleSelection(entry.clone())),
                ))
            })
            .into()
    }

    fn view_queue(&self) -> Element<Message> {
        scrollable(
            self.candidates
                .iter()
                .filter(|(_, selected)| **selected)
                .fold(Column::new().spacing(2), |column, (entry, _)| {
                    let style = match self.queue.get(entry) {
                        Some(Status::InProgress) => {
                            theme::button::entry_warning
                        }
                        Some(Status::Success) => theme::button::entry_success,
                        Some(Status::Error(_)) => theme::button::entry_danger,
                        None => theme::button::entry_queued,
                    };
                    column.push(
                        button(text(entry.name.clone()))
                            .style(style)
                            .on_press_maybe(self.enabled.then_some(
                                Message::ToggleSelection(entry.clone()),
                            ))
                            .width(Length::Fill),
                    )
                }),
        )
        .into()
    }

    fn view_left(&self) -> Column<Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .height(Length::Shrink)
                    .push(
                        button(text("\u{E802}").font(fonts::ICONS_FONT))
                            .on_press_maybe(
                                self.enabled.then_some(Message::PickSource),
                            ),
                    )
                    .push(heading("from"))
                    .push(self.view_source_info()),
            )
            .push(
                checkbox("trickset", self.components.trickset).on_toggle_maybe(
                    self.enabled.then_some(Message::ToggleTricksetComponent),
                ),
            )
            .push(checkbox("scales", self.components.scales).on_toggle_maybe(
                self.enabled.then_some(Message::ToggleScalesComponent),
            ))
            .push(Row::new().height(Length::Fill).align_y(Vertical::Bottom))
    }

    fn view_center(&self) -> Column<Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .height(Length::Shrink)
                    .push(
                        button(text("\u{E802}").font(fonts::ICONS_FONT))
                            .on_press_maybe(
                                self.enabled.then_some(Message::PickSavesDir),
                            ),
                    )
                    .push(heading("to"))
                    .push(self.view_saves_dir()),
            )
            .push(
                button(
                    text(match self.default_selection {
                        true => "deselect all",
                        false => "select all",
                    })
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    }),
                )
                .style(theme::button::secondary)
                .on_press_maybe(
                    self.enabled.then_some(Message::ToggleSelectAll),
                )
                .width(Length::Fill),
            )
            .push(scrollable(self.view_entries()))
    }

    fn view_right(&self) -> Column<Message> {
        Column::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .align_y(Vertical::Center)
                    .push(
                        button(text("\u{E803}").font(fonts::ICONS_FONT))
                            .on_press_maybe(
                                self.enabled.then_some(Message::Start),
                            ),
                    )
                    .push(heading("queue")),
            )
            .push(self.view_queue())
    }

    pub fn view(&self) -> Element<Message> {
        Row::new()
            .push(self.view_left().width(Length::Fill))
            .push(self.view_center().width(Length::Fill))
            .push(self.view_right().width(Length::Fill))
            .width(Length::Fill)
            .spacing(10)
            .into()
    }
}

async fn pick_source() -> Option<PathBuf> {
    Some(
        AsyncFileDialog::new()
            .add_filter("CAS file (.SKA)", &["SKA"])
            .pick_file()
            .await?
            .path()
            .into(),
    )
}

async fn load_source(entry: save::Entry) -> Result<thug_pro::Cas> {
    let content = tokio::spawn(async move { thug_pro::Ska::read_from(&entry) })
        .await
        .map_err(|_| Error::Task)??;

    let cas = thug_pro::Cas::try_from(content)?;

    Ok(cas)
}

async fn load_lut() -> Result<Lut> {
    let content = tokio::spawn(async move { Lut::thug_pro() })
        .await
        .map_err(|_| Error::Task)??;

    Ok(content)
}

async fn load_candidates(
    saves_dir: Option<impl AsRef<Path>>,
    selections: Selections,
    default_selection: bool,
) -> Result<IndexMap<save::Entry, bool>> {
    let saves_dir = saves_dir.ok_or(Error::NoSavesDir)?;
    let entries = save::find_entries(saves_dir)?;

    log::info!("found {} saves", entries.len());

    let candidates = entries
        .into_iter()
        .map(|entry| {
            (
                entry.clone(),
                *selections.get(&entry.name).unwrap_or(&default_selection),
            )
        })
        .collect::<IndexMap<_, _>>();

    Ok(candidates)
}

fn make_transform(
    source: &thug_pro::cas::Cas,
    components: Components,
) -> thug_pro::cas::Cas {
    thug_pro::cas::Cas {
        summary: source.summary.clone(),
        data: thug_pro::cas::Data {
            // CustomSkater
            custom_skater: source.data.custom_skater.clone().map(
                move |custom_skater| {
                    thug_pro::cas::CustomSkater {
                        // CustomSkater.custom
                        custom: custom_skater.custom.map(move |custom| {
                            thug_pro::cas::Custom {
                                // CustomSkater.custom.info
                                info: custom.info.map(|info| {
                                    thug_pro::cas::Info {
                                        // CustomSkater.custom.info.trick_mapping
                                        trick_mapping: components
                                            .trickset
                                            .then_some(info.trick_mapping)
                                            .flatten(),
                                        // CustomSkater.custom.info.specials
                                        specials: components
                                            .trickset
                                            .then_some(info.specials)
                                            .flatten(),
                                    }
                                }),
                                // CustomSkater.custom.appearance (scales group)
                                scales: components
                                    .scales
                                    .then_some(custom.scales)
                                    .flatten(),
                            }
                        }),
                    }
                },
            ),
            story_skater: source.data.story_skater.clone().map(
                move |story_skater| thug_pro::cas::StorySkater {
                    tricks: components
                        .trickset
                        .then_some(story_skater.tricks)
                        .flatten(),
                },
            ),
        },
    }
}

async fn pre_process<P: AsRef<Path>>(
    backup_dir: P,
    source: thug_pro::Cas,
    components: Components,
) -> Result<(Arc<thug_pro::Cas>, PathBuf)> {
    let backup_dir = backup_dir.as_ref();
    fs::create_dir_all(backup_dir).await?;
    let transform = Arc::new(make_transform(&source, components));

    tasks::write(
        Arc::clone(&transform),
        backup_dir.join("transform.ron"),
        Format::Ron,
    )
    .await?;

    Ok((Arc::clone(&transform), PathBuf::from(backup_dir)))
}

async fn process_entry<P: AsRef<Path>>(
    entry: save::Entry,
    backup_dir: P,
    transform: Arc<thug_pro::Cas>,
) -> Result<()> {
    let backup_dir = backup_dir.as_ref();

    let backup_entry = entry.with_dir(backup_dir);
    let backup_filepath = backup_entry.filepath();

    let filepath = entry.filepath();

    log::info!("backing up {:?} to {:?}", filepath, backup_filepath);
    fs::copy(&filepath, &backup_filepath).await?;

    let mut ska = thug_pro::Ska::read_from(&entry)?;

    transform.modify(&mut ska)?;
    ska.write_to(&entry)?;

    log::info!("overwrote save at {:?}", filepath);

    entry.overwrite_metadata()?;

    Ok(())
}

async fn pick_saves_dir() -> Option<PathBuf> {
    Some(AsyncFileDialog::new().pick_folder().await?.path().into())
}
