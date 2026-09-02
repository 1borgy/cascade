use std::{
    borrow::Cow,
    fmt::Debug,
    fs, io,
    path::{Path, PathBuf},
    result,
};

use cascade_save as save;
use cascade_thug2 as thug2;
use iced::Task;
use indexmap::IndexMap;
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

use crate::{paths, tasks, Element, Row};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("paths error: {0}")]
    Path(#[from] paths::Error),

    #[error("tasks error: {0}")]
    Tasks(#[from] tasks::Error),

    #[error("thug2 error: {0}")]
    Thug2(#[from] thug2::Error),

    #[error("save error: {0}")]
    Save(#[from] save::Error),

    #[error("error spawning task")]
    Task,

    #[error("no saves dir set")]
    NoSavesDir,

    #[error("ron deserialization error: {0}")]
    Ron(#[from] SpannedError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Deserialize)]
struct State<'a> {
    from: Option<PathBuf>,
    to_dir: Option<PathBuf>,
    #[serde(borrow)]
    selections: IndexMap<Cow<'a, str>, bool>,
    flags: cascade_core::Flags,
    default_selection: bool,
}

impl Default for State<'_> {
    fn default() -> Self {
        Self {
            from: Default::default(),
            to_dir: Default::default(),
            selections: Default::default(),
            flags: Default::default(),
            default_selection: Default::default(),
        }
    }
}

impl<'a> State<'a> {
    fn read(path: &'a Path) -> Result<Self> {
        let file = fs::File::open(&path)?;

        log::info!("reading state from {:?}", path);

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

struct StateHandle<'a> {
    state: State<'a>,
    path: &'a Path,
}

impl<'a> StateHandle<'a> {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
    }
}

#[derive(Debug)]
struct States<'a> {
    thug2: State<'a>,
    thaw: State<'a>,
}

pub struct Dashboard {}

impl Dashboard {
    pub fn new() -> Self {
        Dashboard {}
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }

    pub fn view(&self) -> Element<'_, Message> {
        Row::new().into()
    }
}
