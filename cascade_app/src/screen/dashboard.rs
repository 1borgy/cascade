use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    result,
    sync::Arc,
};

use cascade::{
    lut::{self, Lut},
    qb,
    save::{self, thug_pro::cas},
};
use iced::{
    widget::{self, button, checkbox, scrollable, text, Column, Row, Space},
    Alignment, Element, Length, Task,
};
use rfd::AsyncFileDialog;
use tokio::fs;

use crate::{
    config::{Format, Selections},
    paths, tasks,
    widget::heading,
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

    #[error("cas error: {0}")]
    Cas(#[from] cas::Error),

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
pub struct SelectableEntry {
    entry: save::Entry,
    selected: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Flags {
    trickset: bool,
    scales: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    PickSource,

    SourcePicked(Option<PathBuf>),
    SourceCopied(Result<()>),
    SourceLoaded(Result<Arc<cas::Cas>>),
    SourceDumped(Result<usize>),

    ToggleSelection(String),
    LoadedEntries(Result<BTreeMap<String, SelectableEntry>>),

    ToggleTrickset(bool),
    ToggleScales(bool),

    LoadedLut(Result<Lut>),

    Go,
    Done(Result<()>),
}

#[derive(Debug, Clone)]
pub enum Event {
    SelectionsUpdated(Selections),
}

pub struct Dashboard {
    source_path: PathBuf,
    source_dump_path: PathBuf,
    backup_dir: PathBuf,

    source: Option<Arc<cas::Cas>>,

    saves_dir: Option<PathBuf>,
    entries: BTreeMap<String, SelectableEntry>,
    default_selection: bool,

    flags: Flags,
    warning_message: Option<String>,

    lut: Option<Lut>,
}

impl Dashboard {
    pub fn new(
        cascade_dir: impl AsRef<Path>,
        saves_dir: Option<PathBuf>,
        backup_dir: PathBuf,
        default_selection: bool,
        selections: Selections,
    ) -> (Self, Task<Message>) {
        let cascade_dir = cascade_dir.as_ref();

        let source_path = paths::source(cascade_dir);
        let source_dump_path = paths::source_dump(cascade_dir);

        (
            Dashboard {
                source_path: source_path.clone(),
                source_dump_path,
                backup_dir,
                source: None,
                saves_dir: saves_dir.clone(),
                entries: BTreeMap::new(),
                default_selection,
                flags: Flags::default(),
                warning_message: None,
                lut: None,
            },
            Task::batch(vec![
                Task::perform(load_source(source_path), Message::SourceLoaded),
                Task::perform(
                    load_entries(saves_dir, selections, default_selection),
                    Message::LoadedEntries,
                ),
                Task::perform(load_lut(), Message::LoadedLut),
            ]),
        )
    }

    fn selections(&self) -> Selections {
        self.entries
            .iter()
            .map(|(name, selection)| (name.clone(), selection.selected))
            .collect::<HashMap<String, bool>>()
            .into()
    }

    pub fn set_saves_dir(
        &mut self,
        saves_dir: impl AsRef<Path>,
    ) -> Task<Message> {
        self.saves_dir = Some(saves_dir.as_ref().into());

        Task::perform(
            load_entries(
                self.saves_dir.clone(),
                self.selections(),
                self.default_selection,
            ),
            Message::LoadedEntries,
        )
    }

    #[expect(dead_code)]
    pub fn set_default_selection(&mut self, default_selection: bool) {
        self.default_selection = default_selection;
    }

    fn set_warning(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        log::warn!("{}", msg);
        self.warning_message = Some(msg);
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::PickSource => {
                (Task::perform(pick_source(), Message::SourcePicked), None)
            }

            Message::SourcePicked(Some(path)) => (
                Task::perform(
                    copy_file(path, self.source_path.clone()),
                    Message::SourceCopied,
                ),
                None,
            ),
            Message::SourcePicked(None) => (Task::none(), None),

            Message::SourceDumped(Ok(_data)) => (Task::none(), None),
            Message::SourceDumped(Err(err)) => {
                self.set_warning(format!("error dumping source: {}", err));
                (Task::none(), None)
            }

            Message::SourceCopied(Ok(_)) => (
                Task::perform(
                    load_source(self.source_path.clone()),
                    Message::SourceLoaded,
                ),
                None,
            ),
            Message::SourceCopied(Err(err)) => {
                self.set_warning(format!("error copying source: {}", err));
                (Task::none(), None)
            }

            Message::SourceLoaded(Ok(content)) => {
                self.source = Some(Arc::clone(&content));

                match cas::Data::try_from(content.data.clone()) {
                    Ok(data) => (
                        Task::perform(
                            tasks::write_serializable(
                                data,
                                self.source_dump_path.clone(),
                                Format::Ron,
                            ),
                            |result| {
                                Message::SourceDumped(
                                    result.map_err(Error::from),
                                )
                            },
                        ),
                        None,
                    ),
                    Err(err) => {
                        self.set_warning(format!(
                            "error loading data: {}",
                            err
                        ));
                        (Task::none(), None)
                    }
                }
            }
            Message::SourceLoaded(Err(err)) => {
                self.set_warning(format!("error loading source: {}", err));
                (Task::none(), None)
            }

            Message::LoadedEntries(Ok(entries)) => {
                self.entries = entries;
                (Task::none(), None)
            }
            Message::LoadedEntries(Err(err)) => {
                self.set_warning(format!("error loading selections: {}", err));
                (Task::none(), None)
            }

            Message::LoadedLut(Ok(lut)) => {
                self.lut = Some(lut);
                (Task::none(), None)
            }
            Message::LoadedLut(Err(err)) => {
                self.set_warning(format!("error loading LUT: {}", err));
                (Task::none(), None)
            }

            Message::ToggleSelection(name) => {
                if let Some(entry) = self.entries.get_mut(&name) {
                    entry.selected = !entry.selected;
                }

                (
                    Task::none(),
                    Some(Event::SelectionsUpdated(self.selections())),
                )
            }

            Message::ToggleTrickset(selected) => {
                self.flags.trickset = selected;
                (Task::none(), None)
            }

            Message::ToggleScales(selected) => {
                self.flags.scales = selected;
                (Task::none(), None)
            }

            Message::Go => match &self.source {
                Some(source) => (
                    Task::perform(
                        go(
                            self.backup_dir.clone(),
                            self.entries
                                .values()
                                .filter_map(|entry| {
                                    entry
                                        .selected
                                        .then_some(entry.entry.clone())
                                })
                                .collect::<Vec<_>>(),
                            Arc::clone(source),
                            self.flags.clone(),
                        ),
                        Message::Done,
                    ),
                    None,
                ),
                None => (Task::none(), None),
            },
            Message::Done(Err(err)) => {
                self.set_warning(format!("done error: {}", err));

                (Task::none(), None)
            }
            Message::Done(Ok(_)) => {
                self.set_warning("done");

                (Task::none(), None)
            }
        }
    }

    fn view_source_info(&self) -> Element<Message> {
        if let Some(source) = &self.source {
            // TODO: clean up :(
            let filename = source
                .summary
                .0
                .get(qb::Id::Checksum(3287553690))
                .map(|symbol| match &symbol.value {
                    qb::Value::String(contents) => {
                        log::info!("{:?}", contents);
                        std::str::from_utf8(&contents).unwrap_or("???")
                    }
                    _ => "???",
                })
                .unwrap_or("???");

            log::info!("filename: ^{}$", filename);

            let gender = source
                .summary
                .0
                .get(qb::Id::Compressed8(220))
                .map(|symbol| match &symbol.value {
                    qb::Value::U8(1) => "male",
                    qb::Value::ZeroInt => "female",
                    _ => "???",
                })
                .unwrap_or("???");

            Column::new()
                .push(text(format!("filename: {}", filename)).size(12))
                .push(text(format!("gender: {}", gender)).size(12))
                .push(text(format!("\ntodo: idk add more fields here")).size(12))
                .into()
        } else {
            text("no source loaded").into()
        }
    }

    fn view_selection_button<'a>(
        &self,
        label: &'a String,
        selected: bool,
    ) -> Element<'a, Message> {
        let checkbox = checkbox("", selected)
            .style(match selected {
                true => widget::checkbox::primary,
                false => widget::checkbox::secondary,
            })
            .on_toggle(move |_| Message::ToggleSelection(label.clone()));

        button(Row::new().push(checkbox).push(label.as_str()))
            .style(match selected {
                true => widget::button::primary,
                false => widget::button::secondary,
            })
            .on_press(Message::ToggleSelection(label.clone()))
            .width(Length::Fill)
            .into()
    }
    fn view_selection_list(&self) -> Element<Message> {
        scrollable(self.entries.iter().fold(
            Column::new().spacing(2),
            |column, (filename, save)| {
                column.push(self.view_selection_button(filename, save.selected))
            },
        ))
        .into()
    }

    fn view_flag<'a, F>(
        &self,
        label: impl Into<String>,
        is_selected: bool,
        message: F,
    ) -> Element<'a, Message>
    where
        F: 'a + Fn(bool) -> Message,
    {
        checkbox(label, is_selected).on_toggle(message).into()
    }

    fn view_left(&self) -> Column<Message> {
        Column::new()
            .align_x(Alignment::Start)
            .spacing(10)
            .push(heading("source"))
            .push(button("set source cas").on_press(Message::PickSource))
            .push(self.view_source_info())
   }

    fn view_center(&self) -> Column<Message> {
        Column::new()
            .align_x(Alignment::Center)
            .spacing(10)
            .push(heading("select"))
            .push(self.view_selection_list())
    }

    fn view_right(&self) -> Column<Message> {
        Column::new()
            .align_x(Alignment::End)
            .push(heading("copy"))
            .push(Space::new(10, 10))
            .push(self.view_flag(
                "trickset",
                self.flags.trickset,
                Message::ToggleTrickset,
            ))
            .push(self.view_flag(
                "scales",
                self.flags.scales,
                Message::ToggleScales,
            ))
            .push(button("go!").on_press(Message::Go))
            .push_maybe(
                self.warning_message.clone().map(|msg| text(msg).size(12)),
            )
    }

    pub fn view(&self) -> Element<Message> {
        Row::new()
            .push(self.view_left().width(Length::Fill))
            .push(self.view_center().width(Length::Fill))
            .push(self.view_right().width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
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

async fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    tokio::fs::copy(&from, &to).await?;
    Ok(())
}

async fn load_source(path: impl AsRef<Path>) -> Result<Arc<cas::Cas>> {
    let entry = save::Entry::at_path(path)?;

    let content = tokio::spawn(async move { entry.load_content() })
        .await
        .map_err(|_| Error::Task)??;

    let cas = cas::Cas::try_from(content)?;

    Ok(Arc::new(cas))
}

async fn load_lut() -> Result<Lut> {
    let content = tokio::spawn(async move { Lut::thug_pro() })
        .await
        .map_err(|_| Error::Task)??;

    Ok(content)
}

async fn load_entries(
    saves_dir: Option<impl AsRef<Path>>,
    selections: Selections,
    default_selection: bool,
) -> Result<BTreeMap<String, SelectableEntry>> {
    let saves_dir = saves_dir.ok_or(Error::NoSavesDir)?;
    let entries = save::load_entries(saves_dir)?;

    log::info!("found {} saves", entries.len());

    Ok(entries
        .into_iter()
        .map(|save| {
            let selected =
                *selections.get(&save.name).unwrap_or(&default_selection);

            (
                save.name.clone(),
                SelectableEntry {
                    selected,
                    entry: save,
                },
            )
        })
        .collect())
}

fn make_transform(source: &cas::Cas, flags: Flags) -> cas::Cas {
    cas::Cas {
        header: source.header.clone(),
        summary: source.summary.clone(),
        data: cas::Data {
            // CustomSkater
            custom_skater: source.data.custom_skater.clone().map(
                move |custom_skater| {
                    cas::CustomSkater {
                        // CustomSkater.custom
                        custom: custom_skater.custom.map(move |custom| {
                            cas::Custom {
                                // CustomSkater.custom.info
                                info: custom.info.map(|info| cas::Info {
                                    // CustomSkater.custom.info.trick_mapping
                                    trick_mapping: flags
                                        .trickset
                                        .then_some(info.trick_mapping)
                                        .flatten(),
                                    // CustomSkater.custom.info.specials
                                    specials: flags
                                        .trickset
                                        .then_some(info.specials)
                                        .flatten(),
                                }),
                                // CustomSkater.custom.appearance (scales group)
                                scales: flags
                                    .scales
                                    .then_some(custom.scales)
                                    .flatten(),
                            }
                        }),
                    }
                },
            ),
            story_skater: source.data.story_skater.clone().map(
                move |story_skater| cas::StorySkater {
                    tricks: flags
                        .trickset
                        .then_some(story_skater.tricks)
                        .flatten(),
                },
            ),
        },
    }
}

async fn backup<P: AsRef<Path>>(
    backup_dir: P,
    entries: &Vec<save::Entry>,
) -> Result<()> {
    let backup_dir = backup_dir.as_ref();

    let datetime = time::OffsetDateTime::now_local().unwrap_or({
        log::warn!("could not get local timezone; using utc");
        time::OffsetDateTime::now_utc()
    });

    let subdir_name = format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        datetime.year(),
        u8::from(datetime.month()),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second()
    );

    let mut subdir = PathBuf::from(backup_dir);
    subdir.push(subdir_name);

    fs::create_dir_all(&subdir).await?;

    for entry in entries.into_iter() {
        let backup_path = subdir.join(entry.filename());
        fs::copy(entry.filepath(), backup_path).await?;
    }

    Ok(())
}

async fn go(
    backup_dir: impl AsRef<Path>,
    entries: impl IntoIterator<Item = save::Entry> + Clone,
    source: Arc<cas::Cas>,
    flags: Flags,
) -> Result<()> {
    let entries = entries.into_iter().collect::<Vec<_>>();

    backup(backup_dir, &entries).await?;

    let transform = Arc::new(make_transform(source.as_ref(), flags));
    let transform_path = paths::transform_dump(paths::cascade_dir()?);

    tasks::write_serializable(&transform, transform_path, Format::Ron).await?;

    for entry in entries.into_iter() {
        let transform = Arc::clone(&transform);

        let name = entry.name.clone();

        let result =
            tokio::spawn(async move { process_entry(&entry, transform).await })
                .await
                .map_err(|_| Error::Task)?;

        match result {
            Ok(_) => {
                // TODO: idk report status
                log::info!("processed entry {}", name);
            }
            Err(err) => {
                log::error!("error processing entry {}", err);
            }
        }
    }
    Ok(())
}

async fn process_entry(
    entry: &save::Entry,
    transform: Arc<cas::Cas>,
) -> Result<()> {
    let mut content = entry.load_content()?;

    // let entry_cas = cas::Cas::try_from(content.clone())?;
    // tasks::write_serializable(
    //     &entry_cas,
    //     entry.filepath().with_added_extension("ron"),
    //     Format::Ron,
    // )
    // .await?;

    transform.modify(&mut content)?;

    entry.write_content(&content)?;
    entry.overwrite_metadata()?;

    // let out_entry = entry.with_name(format!("{}.out", entry.name));
    // let out_entry_cas = cas::Cas::try_from(content.clone())?;
    // tasks::write_serializable(
    //     &out_entry_cas,
    //     out_entry.filepath().with_added_extension("ron"),
    //     Format::Ron,
    // )
    // .await?;

    Ok(())
}
