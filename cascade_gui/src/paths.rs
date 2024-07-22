use std::{backtrace::Backtrace, env, fs, io, path::PathBuf};

use enum_iterator::{all, Sequence};
use thiserror::Error;

const CASCADE_CONFIG_FILENAME: &'static str = "cascade.toml";

#[derive(Error, Debug)]
pub enum PathError {
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error("no home directory was found")]
    NoHomeDir,

    #[error("no thugpro directory was found")]
    NoThugProDir,

    #[error("no thugpro saves directory was found")]
    NoThugProSavesDir,

    #[error("no cascade config directory was found")]
    NoConfigDir,
}

fn detect_local_appdata_dir() -> Result<PathBuf, PathError> {
    Ok(directories::BaseDirs::new()
        .map(|user_dirs| PathBuf::from(user_dirs.data_local_dir()))
        .ok_or(PathError::NoHomeDir)?)
}

pub fn detect_thugpro_dir() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/
    let config_dir = detect_local_appdata_dir().map(|mut config_dir| {
        config_dir.push("THUG Pro");
        config_dir
    })?;

    // TODO: also check that exe is in this dir
    config_dir
        .is_dir()
        .then(|| config_dir)
        .ok_or(PathError::NoThugProDir)
}

pub fn detect_thugpro_saves_dir() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/Save/
    let mut saves_dir = detect_thugpro_dir()?;
    saves_dir.push("Save");

    saves_dir
        .is_dir()
        .then(|| saves_dir)
        .ok_or(PathError::NoThugProSavesDir)
}

#[derive(Debug, Default, Sequence)]
enum CascadeDirLocation {
    #[default]
    LocalAppdata,
    Portable,
    ThugPro, // Legacy cascade install (<0.3.0)
}

impl CascadeDirLocation {
    fn detect() -> Result<Self, PathError> {
        all::<CascadeDirLocation>()
            .filter(|location| location.config_exists())
            .next()
            .ok_or(PathError::NoConfigDir)
    }

    fn try_get_or_make_dir(&self) -> Result<PathBuf, PathError> {
        let dir = match self {
            CascadeDirLocation::LocalAppdata => {
                // %localappdata%/cascade
                let config_dir =
                    detect_local_appdata_dir().map(|mut config_dir| {
                        config_dir.push("cascade");
                        config_dir
                    })?;

                config_dir
            }
            CascadeDirLocation::Portable => env::current_dir()?,
            CascadeDirLocation::ThugPro => {
                // %localappdata%/THUG Pro/.cascade
                let cascade_dir =
                    detect_thugpro_dir().map(|mut config_dir| {
                        config_dir.push(".cascade");
                        config_dir
                    })?;

                cascade_dir
            }
        };

        if !dir.is_dir() {
            fs::create_dir_all(&dir)?;
        }

        Ok(dir)
    }

    fn try_get_config_path(&self) -> Result<PathBuf, PathError> {
        self.try_get_or_make_dir().map(|mut config_dir| {
            config_dir.push(CASCADE_CONFIG_FILENAME);
            config_dir
        })
    }

    fn config_exists(&self) -> bool {
        self.try_get_config_path()
            .is_ok_and(|filepath| filepath.exists())
    }
}

pub fn detect_cascade_dir() -> Result<PathBuf, PathError> {
    let dir = CascadeDirLocation::detect()
        .unwrap_or_default()
        .try_get_or_make_dir()?;

    log::info!("using cascade dir {:?}", dir);

    Ok(dir)
}

pub fn get_config_path() -> Result<PathBuf, PathError> {
    detect_cascade_dir().map(|mut config_path| {
        config_path.push(CASCADE_CONFIG_FILENAME);
        config_path
    })
}

pub fn default_backup_path() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/backup/
    let mut backup_path = detect_cascade_dir()?;
    backup_path.push("backup");

    if !backup_path.is_dir() {
        fs::create_dir_all(&backup_path)?;
    }

    Ok(backup_path)
}

pub fn default_trickset_path() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/trickset.SKA
    let mut trickset_path = detect_cascade_dir()?;
    trickset_path.push("trickset.SKA");

    Ok(trickset_path)
}
