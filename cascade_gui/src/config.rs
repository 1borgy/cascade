use std::{
    backtrace::Backtrace,
    fmt, fs,
    io::{self, Write},
    path::PathBuf,
};

use enum_iterator::Sequence;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::paths;

#[derive(Error, Debug)]
pub enum ConfigError {
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

    #[error("an error finding paths occurred: {source}")]
    Paths {
        #[from]
        source: paths::PathError,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CascadeConfig {
    pub paths: CascadePaths,
    pub theme: CascadeTheme,
}

impl CascadeConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = paths::get_config_path()?;
        let file = fs::File::open(&path)?;

        log::info!("reading config from {:?}", path);

        let contents = io::read_to_string(file)?;
        let config = toml::from_str(contents.as_str())?;

        Ok(config)
    }

    pub fn write(&self) -> Result<(), ConfigError> {
        let path = paths::get_config_path()?;
        let mut file = fs::File::create(&path)?;

        log::info!("writing config to {:?}", path);

        let contents = toml::to_string(&self)?;
        write!(file, "{}", contents)?;

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
    pub trickset_path: Option<PathBuf>,
}

impl Default for CascadePaths {
    fn default() -> Self {
        let saves_dir = match paths::detect_thugpro_saves_dir() {
            Ok(dir) => {
                log::info!("autodetected thug pro saves dir at {:?}", dir);
                Some(dir)
            }
            Err(err) => {
                log::warn!("could not autodetect thug pro saves dir: {}", err);
                None
            }
        };

        let backup_dir = match paths::default_backup_path() {
            Ok(dir) => {
                log::info!("defaulting to backup directory at {:?}", dir);
                Some(dir)
            }
            Err(err) => {
                log::warn!("could not use default backup dir: {}", err);
                None
            }
        };

        let trickset_path = match paths::default_trickset_path() {
            Ok(path) => {
                log::info!("defaulting to trickset at {:?}", path);
                Some(path)
            }
            Err(err) => {
                log::warn!("could not use default trickset path: {}", err);
                None
            }
        };

        Self {
            saves_dir,
            backup_dir,
            trickset_path,
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
    Dark,
    #[default]
    System,
}

impl CascadeBackground {
    pub fn get_palette(&self) -> CascadePalette {
        match self {
            CascadeBackground::Light => CascadePalette::light(),
            CascadeBackground::Dark => CascadePalette::dark(),
            CascadeBackground::System => {
                // autodetect dark/light theme on system
                let mode = dark_light::detect();

                match mode {
                    dark_light::Mode::Dark => {
                        log::info!("autodetected system dark theme");
                        CascadePalette::dark()
                    }
                    dark_light::Mode::Light => {
                        log::info!("autodetected system light theme");
                        CascadePalette::light()
                    }
                    dark_light::Mode::Default => {
                        log::warn!("could not autodetect system theme; defaulting to light");
                        CascadePalette::light()
                    }
                }
            }
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

impl fmt::Display for CascadeBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CascadeBackground::Light => "light",
                CascadeBackground::Dark => "dark",
                CascadeBackground::System => "system",
            }
        )
    }
}

impl fmt::Display for CascadeColor {
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
