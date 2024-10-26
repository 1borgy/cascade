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
    save::{self, thug_pro},
};
use iced::{
    widget::{self, button, checkbox, scrollable, text, Column, Row},
    Alignment, Element, Length, Task,
};
use rfd::AsyncFileDialog;
use tokio::fs;

use crate::{config::Selections, fonts, paths, tasks, widget::heading};

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
pub struct Entry {
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
    LoadedEntries(Result<BTreeMap<String, Entry>>),
    LoadedSource(Result<Arc<thug_pro::cas::Cas>>),
    LoadedLut(Result<Lut>),

    PickSource,
    SourcePicked(Option<PathBuf>),

    ToggleSelectAll,
    ToggleSelection(String),
    ToggleTricksetFlag(bool),
    ToggleScalesFlag(bool),

    Go,
    Done(Result<()>),
}

#[derive(Debug, Clone)]
pub enum Event {
    SetSourcePath(PathBuf),
    SetDefaultSelection(bool, Selections),
    SetSelections(Selections),
}

pub struct Dashboard {
    backup_dir: PathBuf,

    source_entry: Option<save::Entry>,
    source: Option<Arc<thug_pro::Cas>>,

    saves_dir: Option<PathBuf>,
    entries: BTreeMap<String, Entry>,
    default_selection: bool,

    flags: Flags,

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
                load_entries(saves_dir.clone(), selections, default_selection),
                Message::LoadedEntries,
            ),
            Task::perform(load_lut(), Message::LoadedLut),
        ]);

        let dashboard = Dashboard {
            backup_dir,
            source_entry,
            source: None,
            saves_dir,
            entries: BTreeMap::new(),
            default_selection,
            flags: Flags::default(),
            warning_message: None,
            lut: None,
        };

        (dashboard, tasks)
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
            Message::LoadedSource(Ok(content)) => {
                self.source = Some(Arc::clone(&content));
                (Task::none(), None)
            }
            Message::LoadedSource(Err(err)) => {
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
                        self.set_warning(format!(
                            "error picking source: {err}"
                        ));
                        (Task::none(), None)
                    }
                }
            }
            Message::SourcePicked(None) => (Task::none(), None),

            Message::ToggleSelectAll => {
                self.default_selection = !self.default_selection;

                for (_, entry) in self.entries.iter_mut() {
                    entry.selected = self.default_selection;
                }

                (
                    Task::none(),
                    Some(Event::SetDefaultSelection(
                        self.default_selection,
                        self.selections(),
                    )),
                )
            }

            Message::ToggleSelection(name) => {
                if let Some(entry) = self.entries.get_mut(&name) {
                    entry.selected = !entry.selected;
                }

                (Task::none(), Some(Event::SetSelections(self.selections())))
            }

            Message::ToggleTricksetFlag(selected) => {
                self.flags.trickset = selected;
                (Task::none(), None)
            }

            Message::ToggleScalesFlag(selected) => {
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
                .push(
                    text(format!("\ntodo: idk add more fields here")).size(12),
                )
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

    fn view_entries(&self) -> Element<Message> {
        self.entries
            .iter()
            .fold(Column::new().spacing(2), |column, (filename, save)| {
                column.push(self.view_selection_button(filename, save.selected))
            })
            .into()
    }

    fn view_queue(&self) -> Element<Message> {
        scrollable(self.entries.values().filter(|entry| entry.selected).fold(
            Column::new().spacing(2),
            |column, entry| {
                column.push(
                    self.view_selection_button(
                        &entry.entry.name,
                        entry.selected,
                    ),
                )
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
            .push_maybe(
                self.warning_message.clone().map(|msg| text(msg).size(12)),
            )
    }

    fn view_center(&self) -> Column<Message> {
        Column::new()
            .align_x(Alignment::Center)
            .spacing(10)
            .push(heading("select"))
            .push(
                button(
                    text(match self.default_selection {
                        true => "deselect all",
                        false => "select all",
                    })
                    .font(fonts::IOSEVKA_BOLD),
                )
                .on_press(Message::ToggleSelectAll)
                .width(Length::Fill),
            )
            .push(scrollable(self.view_entries()))
    }

    fn view_right(&self) -> Column<Message> {
        Column::new()
            .align_x(Alignment::End)
            .spacing(10)
            .push(heading("copy"))
            .push(self.view_flag(
                "trickset",
                self.flags.trickset,
                Message::ToggleTricksetFlag,
            ))
            .push(self.view_flag(
                "scales",
                self.flags.scales,
                Message::ToggleScalesFlag,
            ))
            .push(button("go!").on_press(Message::Go))
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

async fn load_source(entry: save::Entry) -> Result<Arc<thug_pro::Cas>> {
    let content = tokio::spawn(async move { thug_pro::Ska::read_from(&entry) })
        .await
        .map_err(|_| Error::Task)??;

    let cas = thug_pro::Cas::try_from(content)?;

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
) -> Result<BTreeMap<String, Entry>> {
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
                Entry {
                    selected,
                    entry: save,
                },
            )
        })
        .collect())
}

fn make_transform(
    source: &thug_pro::cas::Cas,
    flags: Flags,
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
                                        trick_mapping: flags
                                            .trickset
                                            .then_some(info.trick_mapping)
                                            .flatten(),
                                        // CustomSkater.custom.info.specials
                                        specials: flags
                                            .trickset
                                            .then_some(info.specials)
                                            .flatten(),
                                    }
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
                move |story_skater| thug_pro::cas::StorySkater {
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
    source: Arc<thug_pro::cas::Cas>,
    flags: Flags,
) -> Result<()> {
    let entries = entries.into_iter().collect::<Vec<_>>();

    backup(backup_dir, &entries).await?;

    let transform = Arc::new(make_transform(source.as_ref(), flags));

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
    transform: Arc<thug_pro::Cas>,
) -> Result<()> {
    let mut ska = thug_pro::Ska::read_from(entry)?;

    transform.modify(&mut ska)?;
    ska.write_to(entry)?;

    entry.overwrite_metadata()?;

    Ok(())
}
