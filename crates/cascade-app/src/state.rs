use std::{
    collections::HashMap,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    Result,
    paths::{self, Paths},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Game {
    Thps3,
    Thps4,
    Thug,
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
                Game::Thps3 => "THPS3",
                Game::Thps4 => "THPS4",
                Game::Thug => "THUG",
                Game::Thug2 => "THUG2",
                Game::Thaw => "THAW",
            }
        )
    }
}

impl Game {
    pub const ALL: &'static [Self] = &[
        Self::Thps3,
        Self::Thps4,
        Self::Thug,
        Self::Thug2,
        Self::Thaw,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f32,
    #[serde(default)]
    pub game: Game,
}

fn default_scale_factor() -> f32 {
    1.
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
    pub selections: HashMap<String, bool>,
}

impl GameState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::File::open(&path)?;

        log::info!("reading game state from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }

    pub fn detect(default_dir: Option<PathBuf>) -> Self {
        if let Some(to_dir) = default_dir {
            Self {
                to_dir: Some(to_dir),
                ..Default::default()
            }
        } else {
            Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameStates {
    pub thps3: GameState,
    pub thps4: GameState,
    pub thug: GameState,
    pub thug2: GameState,
    pub thaw: GameState,
}

impl GameStates {
    pub fn load(paths: &Paths) -> Self {
        let thps3 = GameState::load(&paths.thps3).unwrap_or_default();
        let thps4 = GameState::load(&paths.thps4).unwrap_or_default();
        let thug = GameState::load(&paths.thug).unwrap_or_default();
        let thug2 =
            GameState::load(&paths.thug2).unwrap_or(GameState::detect(paths::detect_thugpro_dir()));
        let thaw = GameState::load(&paths.thaw).unwrap_or_default();

        Self {
            thps3,
            thps4,
            thug,
            thug2,
            thaw,
        }
    }
}

#[derive(Debug, Clone)]
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
