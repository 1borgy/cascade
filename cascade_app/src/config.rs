use std::{
    collections::HashMap,
    fs,
    io::{self},
    ops::Deref,
    path::{Path, PathBuf},
};

use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

use crate::{paths, theme::Theme};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    Ron,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("paths error: {0}")]
    Paths(#[from] paths::Error),

    #[error("ron deserialization error: {0}")]
    Spanned(#[from] SpannedError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

fn default_scale_factor() -> f64 {
    1.
}

fn default_saves_dir() -> Option<PathBuf> {
    match paths::default_saves_dir() {
        Ok(dir) => {
            log::info!("autodetected thug pro saves dir at {:?}", dir);
            Some(dir)
        }
        Err(err) => {
            log::warn!("could not autodetect thug pro saves dir: {}", err);
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub saves_dir: Option<PathBuf>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
    #[serde(default)]
    pub default_selection: bool,
}

impl Default for Config {
    fn default() -> Config {
        Self {
            saves_dir: default_saves_dir(),
            theme: Theme::default(),
            scale_factor: 1.,
            default_selection: false,
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;

        log::info!("reading config from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selections(HashMap<String, bool>);

impl Selections {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;

        log::info!("reading config from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

impl From<HashMap<String, bool>> for Selections {
    fn from(value: HashMap<String, bool>) -> Self {
        Self(value)
    }
}

impl Deref for Selections {
    type Target = HashMap<String, bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//impl FromIterator<(String, bool)> for Selections {
//    fn from_iter<I: IntoIterator<Item = (String, bool)>>(iter: I) -> Self {
//        Selections(HashMap::from_iter(iter))
//    }
//}
