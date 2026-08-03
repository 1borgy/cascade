use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn cwd() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
}

fn local_appdata_dir() -> Option<PathBuf> {
    // %localappdata%/
    directories::BaseDirs::new().map(|user_dirs| user_dirs.data_local_dir().into())
}

fn default_cascade_dir() -> Option<PathBuf> {
    // %localappdata%/cascade/
    if let Some(path) = local_appdata_dir().map(|dir| dir.join("cascade")) {
        if !path.is_dir()
            && let Err(err) = fs::create_dir_all(&path)
        {
            log::warn!("could not create cascade directory: {}", err)
        }
        Some(path)
    } else {
        None
    }
}

pub fn cascade_dir() -> Option<PathBuf> {
    default_cascade_dir().or_else(cwd)
}

pub fn detect_thugpro_dir() -> Option<PathBuf> {
    // %localappdata%/THUG Pro/Save/
    local_appdata_dir()
        .map(|dir| dir.join("THUG Pro").join("Save"))
        .filter(|path| path.is_dir())
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub app: PathBuf,
    pub backup: PathBuf,

    pub thps3: PathBuf,
    pub thps4: PathBuf,
    pub thug: PathBuf,
    pub thug2: PathBuf,
    pub thaw: PathBuf,

    pub theme: PathBuf,
    pub log: PathBuf,
}

impl Paths {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            app: data_dir.join("app.ron"),
            backup: data_dir.join("backup"),
            thps3: data_dir.join("thps3.ron"),
            thps4: data_dir.join("thps4.ron"),
            thug: data_dir.join("thug.ron"),
            thug2: data_dir.join("thug2.ron"),
            thaw: data_dir.join("thaw.ron"),
            theme: data_dir.join("theme.toml"),
            log: data_dir.join("cascade.log"),
        }
    }
}
