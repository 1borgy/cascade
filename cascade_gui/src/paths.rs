use std::{backtrace::Backtrace, fs, io, path::PathBuf};

use thiserror::Error;

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
}

pub fn detect_thugpro_dir() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/
    let config_dir = directories::BaseDirs::new()
        .map(|user_dirs| {
            let local_appdata = user_dirs.data_local_dir();

            let mut config_dir = PathBuf::new();

            config_dir.push(local_appdata);
            config_dir.push("THUG Pro");

            config_dir
        })
        .ok_or(PathError::NoHomeDir)?;

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

pub fn get_cascade_dir() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/
    let mut cascade_dir = detect_thugpro_dir()?;
    cascade_dir.push(".cascade");

    if !cascade_dir.is_dir() {
        fs::create_dir_all(&cascade_dir)?;
    }

    Ok(cascade_dir)
}

pub fn get_config_path() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/config.toml
    let mut config_path = get_cascade_dir()?;
    config_path.push("cascade.toml");

    Ok(config_path)
}

pub fn default_backup_path() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/backup/
    let mut backup_path = get_cascade_dir()?;
    backup_path.push("backup");

    if !backup_path.is_dir() {
        fs::create_dir_all(&backup_path)?;
    }

    Ok(backup_path)
}

pub fn default_trickset_path() -> Result<PathBuf, PathError> {
    // %localappdata%/THUG Pro/.cascade/trickset.SKA
    let mut trickset_path = get_cascade_dir()?;
    trickset_path.push("trickset.SKA");

    Ok(trickset_path)
}
