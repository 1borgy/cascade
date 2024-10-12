use std::{
    env, fs, io,
    path::{Path, PathBuf},
    result,
};

const CONFIG_FILENAME: &'static str = "cascade.ron";
const SELECTIONS_FILENAME: &'static str = "selections.ron";
const SOURCE_FILENAME: &'static str = "source.SKA";
const SOURCE_DUMP_FILENAME: &'static str = "source.ron";
const TRANSFORM_DUMP_FILENAME: &'static str = "transform.ron";
const LOG_FILENAME: &'static str = "cascade.log";

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("an io error occurred: {0}")]
    Io(io::ErrorKind),

    #[error("no home directory was found")]
    NoHomeDir,

    #[error("no thug pro directory was found")]
    NoThugProDir,

    #[error("no thug pro saves directory was found")]
    NoThugProSavesDir,

    #[error("could not determine cwd")]
    Cwd,
}

pub type Result<T, E = Error> = result::Result<T, E>;

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

fn local_appdata_dir() -> Result<PathBuf> {
    // %localappdata%/
    if let Some(user_dirs) = directories::BaseDirs::new() {
        Ok(user_dirs.data_local_dir().into())
    } else {
        Err(Error::NoHomeDir)
    }
}

pub fn default_thug_pro_dir() -> Result<PathBuf> {
    // %localappdata%/THUG Pro/
    let path = local_appdata_dir().map(|dir| dir.join("THUG Pro"))?;

    match path.is_dir() {
        true => Ok(path),
        false => Err(Error::NoThugProDir),
    }
}

pub fn default_saves_dir() -> Result<PathBuf> {
    // %localappdata%/THUG Pro/Save/
    let path = default_thug_pro_dir().map(|dir| dir.join("Save"))?;

    match path.is_dir() {
        true => Ok(path),
        false => Err(Error::NoThugProSavesDir),
    }
}

fn cwd() -> Result<PathBuf> {
    let exe = env::current_exe()?;
    Ok(exe.parent().ok_or(Error::Cwd)?.into())
}

fn portable_dir() -> Option<PathBuf> {
    let cwd = cwd().ok()?;
    let config_path = cwd.join(CONFIG_FILENAME);

    match config_path.is_file() {
        true => Some(cwd.to_path_buf()),
        false => None,
    }
}

fn default_cascade_dir() -> Result<PathBuf> {
    // %localappdata%/cascade/
    let path = local_appdata_dir().map(|dir| dir.join("cascade"))?;

    if !path.is_dir() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}

pub fn cascade_dir() -> Result<PathBuf> {
    match portable_dir() {
        Some(dir) => Ok(dir),
        None => default_cascade_dir().or_else(|_| cwd()),
    }
}

pub fn backup_dir(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/backup/
    cascade_dir.as_ref().join("backup")
}

pub fn config(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/cascade.ron
    cascade_dir.as_ref().join(CONFIG_FILENAME)
}

pub fn selections(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/selections.ron
    cascade_dir.as_ref().join(SELECTIONS_FILENAME)
}

pub fn source(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/source.SKA
    cascade_dir.as_ref().join(SOURCE_FILENAME)
}

pub fn source_dump(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/source.ron
    cascade_dir.as_ref().join(SOURCE_DUMP_FILENAME)
}

pub fn transform_dump(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/transform.ron
    cascade_dir.as_ref().join(TRANSFORM_DUMP_FILENAME)
}

pub fn log(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/cascade.log
    cascade_dir.as_ref().join(LOG_FILENAME)
}
