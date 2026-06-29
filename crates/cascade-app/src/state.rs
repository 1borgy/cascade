use std::{
    collections::HashMap,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, paths::Paths};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Game {
    #[default]
    Thug2,
    Thaw,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Game::Thug2 => "THUG2",
                Game::Thaw => "THAW",
            }
        )
    }
}

impl Game {
    pub const ALL: &'static [Self] = &[Self::Thug2, Self::Thaw];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
    #[serde(default)]
    pub game: Game,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            scale_factor: 1.,
            game: Default::default(),
        }
    }
}

impl AppState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::File::open(&path)?;

        log::info!("reading app state from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub type Selections = HashMap<String, bool>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub from: Option<PathBuf>,
    #[serde(default)]
    pub to_dir: Option<PathBuf>,
    #[serde(default)]
    pub scales: bool,
    #[serde(default)]
    pub trickset: bool,
    #[serde(default)]
    pub default_selection: bool,
    #[serde(default)]
    pub selections: Selections,
}
fn default_scale_factor() -> f64 {
    1.
}

impl GameState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::File::open(&path)?;

        log::info!("reading game state from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

#[derive(Debug)]
pub struct GameStates {
    pub thug2: GameState,
    pub thaw: GameState,
}

impl GameStates {
    pub fn load(paths: &Paths) -> Self {
        let thug2 = GameState::load(&paths.thug2).unwrap_or_default();
        let thaw = GameState::load(&paths.thaw).unwrap_or_default();

        Self { thug2, thaw }
    }
}

#[derive(Debug)]
pub struct State {
    pub app: AppState,
    pub game: GameStates,
}

impl State {
    pub fn load(paths: &Paths) -> Self {
        let app = AppState::load(&paths.app).unwrap_or_default();
        let game = GameStates::load(paths);

        Self { app, game }
    }
}
