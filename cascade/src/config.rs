use std::{
    backtrace::Backtrace, fmt::Display, fs, io, io::Write, path::PathBuf,
};

use enum_iterator::Sequence;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::files;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("no home directory was found")]
    NoHomeDirectory,
    #[error("no thugpro saves folder was found")]
    NoThugproSaves,
    #[error("yaml serde operation failed")]
    YamlSerde {
        #[from]
        source: serde_yaml::Error,
        backtrace: Backtrace,
    },
    #[error("toml deserialization operation failed")]
    TomlDeserialize {
        #[from]
        source: toml::de::Error,
        backtrace: Backtrace,
    },
    #[error("toml serialization operation failed")]
    TomlSerialize {
        #[from]
        source: toml::ser::Error,
        backtrace: Backtrace,
    },
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
}

pub fn get_thugpro_dir() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/
    directories::BaseDirs::new()
        .map(|user_dirs| {
            let local_appdata = user_dirs.data_local_dir();

            let mut config_dir = PathBuf::new();

            config_dir.push(local_appdata);
            config_dir.push("THUG Pro");

            config_dir
        })
        .ok_or(ConfigError::NoHomeDirectory)
}

pub fn get_cascade_dir() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/.cascade/
    let mut cascade_dir = get_thugpro_dir()?;
    cascade_dir.push(".cascade");

    if !cascade_dir.is_dir() {
        fs::create_dir_all(&cascade_dir)?;
    }

    Ok(cascade_dir)
}

fn get_config_path() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/.cascade/config.toml
    let mut config_path = get_cascade_dir()?;
    config_path.push("cascade.toml");

    Ok(config_path)
}

fn get_backup_path() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/.cascade/backup/
    let mut backup_path = get_cascade_dir()?;
    backup_path.push("backup");

    if !backup_path.is_dir() {
        fs::create_dir_all(&backup_path)?;
    }

    Ok(backup_path)
}

fn get_trick_source_path() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/.cascade/tricksource.SKA
    let mut trick_source_path = get_cascade_dir()?;
    trick_source_path.push("tricksource.SKA");

    Ok(trick_source_path)
}

fn get_thugpro_saves_dir() -> Result<PathBuf, ConfigError> {
    // %localappdata%/THUG Pro/Save/
    let mut saves_dir = get_thugpro_dir()?;
    saves_dir.push("Save");

    saves_dir
        .is_dir()
        .then(|| saves_dir)
        .ok_or(ConfigError::NoThugproSaves)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CascadeConfig {
    pub paths: CascadePaths,
    pub theme: CascadeTheme,
}

impl CascadeConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = get_config_path()?;
        let reader = files::load_reader(&path)?;

        log::info!("reading config from {:?}", path);

        let contents = io::read_to_string(reader)?;
        let config = toml::from_str(contents.as_str())?;

        Ok(config)
    }

    pub fn write(&self) -> Result<(), ConfigError> {
        let path = get_config_path()?;
        let mut writer = files::load_writer(&path)?;

        log::info!("writing config to {:?}", path);

        let contents = toml::to_string(&self)?;
        write!(writer, "{}", contents)?;

        Ok(())
    }

    pub fn load_or_create() -> Result<Self, ConfigError> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(err) => match err {
                // if io error is file not found, that's fine, just create a default config
                ConfigError::Io { source, .. }
                    if (source.kind() == io::ErrorKind::NotFound) =>
                {
                    log::info!("no config detected");

                    let config: Self = Default::default();
                    config.write()?;

                    Ok(config)
                }
                _ => Err(err),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadePaths {
    pub saves_dir: Option<PathBuf>,
    pub backup_dir: Option<PathBuf>,
    pub trick_source: Option<PathBuf>,
}

impl Default for CascadePaths {
    fn default() -> Self {
        let saves_folder = match get_thugpro_saves_dir() {
            Ok(folder) => {
                log::info!(
                    "autodetected thug pro saves folder at {:?}",
                    folder
                );
                Some(folder)
            }
            Err(err) => {
                log::warn!(
                    "could not autodetect thug pro saves folder: {}",
                    err
                );
                None
            }
        };

        let backup_folder = match get_backup_path() {
            Ok(folder) => {
                log::info!("defaulting to backup folder at {:?}", folder);
                Some(folder)
            }
            Err(err) => {
                log::warn!("could not use default backup folder: {}", err);
                None
            }
        };

        let trick_source = match get_trick_source_path() {
            Ok(folder) => {
                log::info!("defaulting to trick source at {:?}", folder);
                Some(folder)
            }
            Err(err) => {
                log::warn!("could not use default trick source path: {}", err);
                None
            }
        };

        Self {
            saves_dir: saves_folder,
            backup_dir: backup_folder,
            trick_source,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct CascadePalette {
    pub background: RgbColor,
    pub text: RgbColor,
    pub subtext: RgbColor,
    pub rosewater: RgbColor,
    pub flamingo: RgbColor,
    pub pink: RgbColor,
    pub mauve: RgbColor,
    pub red: RgbColor,
    pub maroon: RgbColor,
    pub peach: RgbColor,
    pub yellow: RgbColor,
    pub green: RgbColor,
    pub teal: RgbColor,
    pub sky: RgbColor,
    pub sapphire: RgbColor,
    pub blue: RgbColor,
    pub lavender: RgbColor,
}

macro_rules! hexcolor {
    ($code:literal) => {{
        let [red, green, blue] = hex!($code);
        RgbColor { red, green, blue }
    }};
}

impl CascadePalette {
    pub fn light() -> Self {
        Self {
            background: hexcolor!("eff1f5"),
            text: hexcolor!("4c4f69"),
            subtext: hexcolor!("6c6f85"),
            rosewater: hexcolor!("dc8a78"),
            flamingo: hexcolor!("dd7878"),
            pink: hexcolor!("ea76cb"),
            mauve: hexcolor!("8839ef"),
            red: hexcolor!("d20f39"),
            maroon: hexcolor!("e64553"),
            peach: hexcolor!("fe640b"),
            yellow: hexcolor!("df8e1d"),
            green: hexcolor!("40a02b"),
            teal: hexcolor!("179299"),
            sky: hexcolor!("04a5e5"),
            sapphire: hexcolor!("209fb5"),
            blue: hexcolor!("1e66f5"),
            lavender: hexcolor!("7287fd"),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: hexcolor!("24273a"),
            text: hexcolor!("cad3f5"),
            subtext: hexcolor!("a5adcb"),
            rosewater: hexcolor!("f4dbd6"),
            flamingo: hexcolor!("f0c6c6"),
            pink: hexcolor!("f5bde6"),
            mauve: hexcolor!("c6a0f6"),
            red: hexcolor!("ed8796"),
            maroon: hexcolor!("ee99a0"),
            peach: hexcolor!("f5a97f"),
            yellow: hexcolor!("eed49f"),
            green: hexcolor!("a6da95"),
            teal: hexcolor!("8bd5ca"),
            sky: hexcolor!("91d7e3"),
            sapphire: hexcolor!("7dc4e4"),
            blue: hexcolor!("8aadf4"),
            lavender: hexcolor!("b7bdf8"),
        }
    }

    pub fn get_color(&self, color: CascadeColor) -> RgbColor {
        match color {
            CascadeColor::Rosewater => self.rosewater,
            CascadeColor::Flamingo => self.flamingo,
            CascadeColor::Pink => self.pink,
            CascadeColor::Mauve => self.mauve,
            CascadeColor::Red => self.red,
            CascadeColor::Maroon => self.maroon,
            CascadeColor::Peach => self.peach,
            CascadeColor::Yellow => self.yellow,
            CascadeColor::Green => self.green,
            CascadeColor::Teal => self.teal,
            CascadeColor::Sky => self.sky,
            CascadeColor::Sapphire => self.sapphire,
            CascadeColor::Blue => self.blue,
            CascadeColor::Lavender => self.lavender,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CascadeTheme {
    pub background: CascadeBackground,
    pub color: CascadeColor,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Sequence, Serialize, Deserialize,
)]
pub enum CascadeBackground {
    Light,
    #[default]
    Dark,
}

impl CascadeBackground {
    pub fn get_palette(&self) -> CascadePalette {
        match self {
            CascadeBackground::Light => CascadePalette::light(),
            CascadeBackground::Dark => CascadePalette::dark(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Sequence, Serialize, Deserialize,
)]
pub enum CascadeColor {
    Rosewater,
    Flamingo,
    Pink,
    Mauve,
    Red,
    Maroon,
    Peach,
    Yellow,
    Green,
    Teal,
    Sky,
    Sapphire,
    #[default]
    Blue,
    Lavender,
}

impl Display for CascadeBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CascadeBackground::Light => "light",
                CascadeBackground::Dark => "dark",
            }
        )
    }
}

impl Display for CascadeColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CascadeColor::Rosewater => "rosewater",
                CascadeColor::Flamingo => "flamingo",
                CascadeColor::Pink => "pink",
                CascadeColor::Mauve => "mauve",
                CascadeColor::Red => "red",
                CascadeColor::Maroon => "maroon",
                CascadeColor::Peach => "peach",
                CascadeColor::Yellow => "yellow",
                CascadeColor::Green => "green",
                CascadeColor::Teal => "teal",
                CascadeColor::Sky => "sky",
                CascadeColor::Sapphire => "sapphire",
                CascadeColor::Blue => "blue",
                CascadeColor::Lavender => "lavender",
            },
        )
    }
}
