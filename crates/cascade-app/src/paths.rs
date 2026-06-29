use std::{
    env, fs, io,
    path::{Path, PathBuf},
    result,
};

const CONFIG_FILENAME: &'static str = "cascade.toml";
const THEME_FILENAME: &'static str = "theme.toml";
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

pub fn default_thugpro_dir() -> Result<PathBuf> {
    // %localappdata%/THUG Pro/
    let path = local_appdata_dir().map(|dir| dir.join("THUG Pro"))?;

    match path.is_dir() {
        true => Ok(path),
        false => Err(Error::NoThugProDir),
    }
}

pub fn default_saves_dir() -> Result<PathBuf> {
    // %localappdata%/THUG Pro/Save/
    let path = default_thugpro_dir().map(|dir| dir.join("Save"))?;

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
pub fn theme(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/theme.toml
    cascade_dir.as_ref().join(THEME_FILENAME)
}

pub fn log(cascade_dir: impl AsRef<Path>) -> PathBuf {
    // %localappdata%/cascade/cascade.log
    cascade_dir.as_ref().join(LOG_FILENAME)
}

// TODO: replace all above
#[derive(Debug)]
pub struct Paths {
    pub app: PathBuf,
    pub thug2: PathBuf,
    pub thaw: PathBuf,
}

impl Paths {
    pub fn new(data_dir: &PathBuf) -> Self {
        Self {
            app: data_dir.join("cascade.ron"),
            thug2: data_dir.join("thug2.ron"),
            thaw: data_dir.join("thaw.ron"),
        }
    }
}
