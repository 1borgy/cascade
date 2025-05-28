use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    result,
    sync::Arc,
};

use cascade_thugpro::{self as thugpro, save};
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
    config::Selections,
    fonts, paths, tasks, theme,
    widget::{self, heading},
    Column, Element, Row,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("paths error: {0}")]
    Path(#[from] paths::Error),

    #[error("tasks error: {0}")]
    Tasks(#[from] tasks::Error),

    #[error("thug pro save error: {0}")]
    ThugPro(#[from] thugpro::Error),

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
    LoadedCandidates(Result<IndexMap<save::Entry, bool>>),
    LoadedSource(Result<thugpro::Cas>),

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
    PreProcessDone(Result<(Arc<thugpro::Cas>, PathBuf)>),
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
    source: Option<thugpro::Cas>,

    candidates: IndexMap<save::Entry, bool>,
    components: Components,
    queue: IndexMap<save::Entry, Status>,

    warning_message: Option<String>,
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
                Some(entry) => Task::perform(load_source(entry.clone()), Message::LoadedSource),
                None => Task::none(),
            },
            Task::perform(
                load_candidates(saves_dir.clone(), selections, default_selection),
                Message::LoadedCandidates,
            ),
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

    pub fn set_saves_dir(&mut self, saves_dir: impl AsRef<Path>) -> Task<Message> {
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

    pub fn update(&mut self, message: Message) -> (Task<Message>, Option<Event>) {
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

            Message::PickSource => (Task::perform(pick_source(), Message::SourcePicked), None),

            Message::SourcePicked(Some(path)) => match save::Entry::at_path(path.clone()) {
                Ok(entry) => {
                    self.source_entry = Some(entry.clone());
                    (
                        Task::perform(load_source(entry), Message::LoadedSource),
                        Some(Event::SetSourcePath(path)),
                    )
                }
                Err(err) => {
                    self.notify(format!("error picking source: {err}"));
                    (Task::none(), None)
                }
            },
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
                                pre_process(backup_dir, source.clone(), self.components),
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
                    .filter_map(|(entry, selected)| selected.then_some(entry.clone()))
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
                                    move |result| Message::EntryProcessed(entry.clone(), result),
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
                        self.notify(format!("error for entry {}: {:?}", entry.name, err));
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
                    &entry.name,
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
                        Some(Status::InProgress) => theme::button::entry_warning,
                        Some(Status::Success) => theme::button::entry_success,
                        Some(Status::Error(_)) => theme::button::entry_danger,
                        None => theme::button::entry_queued,
                    };
                    column.push(
                        button(text(entry.name.clone()))
                            .style(style)
                            .on_press_maybe(
                                self.enabled
                                    .then_some(Message::ToggleSelection(entry.clone())),
                            )
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
                            .on_press_maybe(self.enabled.then_some(Message::PickSource)),
                    )
                    .push(heading("from"))
                    .push(self.view_source_info()),
            )
            .push(
                checkbox("trickset", self.components.trickset)
                    .on_toggle_maybe(self.enabled.then_some(Message::ToggleTricksetComponent)),
            )
            .push(
                checkbox("scales", self.components.scales)
                    .on_toggle_maybe(self.enabled.then_some(Message::ToggleScalesComponent)),
            )
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
                            .on_press_maybe(self.enabled.then_some(Message::PickSavesDir)),
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
                .on_press_maybe(self.enabled.then_some(Message::ToggleSelectAll))
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
                            .on_press_maybe(self.enabled.then_some(Message::Start)),
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

async fn load_source(entry: save::Entry) -> Result<thugpro::Cas> {
    let content = tokio::spawn(async move { thugpro::Save::read_from(&entry) })
        .await
        .map_err(|_| Error::Task)??;

    let cas = thugpro::Cas::try_from(content)?;

    Ok(cas)
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

fn make_transform(source: &thugpro::cas::Cas, components: Components) -> thugpro::cas::Cas {
    let info = &source.data.custom_skater.custom.info;
    let appearance = &source.data.custom_skater.custom.appearance;

    thugpro::cas::Cas {
        summary: source.summary.clone(),
        data: thugpro::cas::Data {
            custom_skater: thugpro::cas::CustomSkater {
                custom: thugpro::cas::Custom {
                    info: thugpro::cas::Info {
                        trick_mapping: components
                            .trickset
                            .then_some(info.trick_mapping.clone())
                            .unwrap_or_default(),
                        specials: components
                            .trickset
                            .then_some(info.specials.clone())
                            .unwrap_or_default(),
                    },
                    appearance: thugpro::cas::Appearance {
                        board_bone_group: components
                            .scales
                            .then_some(appearance.board_bone_group.clone())
                            .unwrap_or_default(),
                        feet_bone_group: components
                            .scales
                            .then_some(appearance.feet_bone_group.clone())
                            .unwrap_or_default(),
                        hands_bone_group: components
                            .scales
                            .then_some(appearance.hands_bone_group.clone())
                            .unwrap_or_default(),
                        head_bone_group: components
                            .scales
                            .then_some(appearance.head_bone_group.clone())
                            .unwrap_or_default(),
                        headtop_bone_group: components
                            .scales
                            .then_some(appearance.headtop_bone_group.clone())
                            .unwrap_or_default(),
                        jaw_bone_group: components
                            .scales
                            .then_some(appearance.jaw_bone_group.clone())
                            .unwrap_or_default(),
                        lower_arm_bone_group: components
                            .scales
                            .then_some(appearance.lower_arm_bone_group.clone())
                            .unwrap_or_default(),
                        lower_leg_bone_group: components
                            .scales
                            .then_some(appearance.lower_leg_bone_group.clone())
                            .unwrap_or_default(),
                        nose_bone_group: components
                            .scales
                            .then_some(appearance.nose_bone_group.clone())
                            .unwrap_or_default(),
                        object_scaling: components
                            .scales
                            .then_some(appearance.object_scaling.clone())
                            .unwrap_or_default(),
                        stomach_bone_group: components
                            .scales
                            .then_some(appearance.stomach_bone_group.clone())
                            .unwrap_or_default(),
                        torso_bone_group: components
                            .scales
                            .then_some(appearance.torso_bone_group.clone())
                            .unwrap_or_default(),
                        upper_arm_bone_group: components
                            .scales
                            .then_some(appearance.upper_arm_bone_group.clone())
                            .unwrap_or_default(),
                        upper_leg_bone_group: components
                            .scales
                            .then_some(appearance.upper_leg_bone_group.clone())
                            .unwrap_or_default(),
                        ..Default::default()
                    },
                },
            },
            story_skater: thugpro::cas::StorySkater {
                tricks: components
                    .trickset
                    .then_some(source.data.story_skater.tricks.clone())
                    .unwrap_or_default(),
            },
        },
    }
}

async fn pre_process<P: AsRef<Path>>(
    backup_dir: P,
    source: thugpro::Cas,
    components: Components,
) -> Result<(Arc<thugpro::Cas>, PathBuf)> {
    let backup_dir = backup_dir.as_ref();
    fs::create_dir_all(backup_dir).await?;
    let transform = Arc::new(make_transform(&source, components));

    // tasks::write(
    //     Arc::clone(&transform),
    //     backup_dir.join("transform.ron"),
    //     Format::Ron,
    // )
    // .await?;

    Ok((Arc::clone(&transform), PathBuf::from(backup_dir)))
}

async fn process_entry<P: AsRef<Path>>(
    entry: save::Entry,
    backup_dir: P,
    transform: Arc<thugpro::Cas>,
) -> Result<()> {
    let backup_dir = backup_dir.as_ref();

    let backup_entry = entry.with_dir(backup_dir);
    let backup_filepath = backup_entry.filepath();

    let filepath = entry.filepath();

    log::info!("backing up {:?} to {:?}", filepath, backup_filepath);
    fs::copy(&filepath, &backup_filepath).await?;

    let mut save = thugpro::Save::read_from(&entry)?;

    transform.modify(&mut save)?;
    save.write_to(&entry)?;

    log::info!("overwrote save at {:?}", filepath);

    entry.overwrite_metadata()?;

    Ok(())
}

async fn pick_saves_dir() -> Option<PathBuf> {
    Some(AsyncFileDialog::new().pick_folder().await?.path().into())
}
