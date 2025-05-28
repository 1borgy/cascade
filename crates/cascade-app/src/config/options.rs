use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{config::Error, paths};

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
    pub source_path: Option<PathBuf>,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
    #[serde(default)]
    pub default_selection: bool,
    #[serde(default)]
    pub scales: bool,
    #[serde(default)]
    pub trickset: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            saves_dir: default_saves_dir(),
            source_path: None,
            scale_factor: 1.,
            default_selection: true,
            scales: false,
            trickset: false,
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;

        log::info!("reading config from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = toml::from_str(contents.as_str())?;

        Ok(config)
    }
}
