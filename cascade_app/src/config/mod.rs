use std::io;

use ron::de::SpannedError;

use crate::paths;

pub mod frappe;
pub mod options;
pub mod selections;
pub mod theme;

pub use options::Config;
pub use selections::Selections;
pub use theme::Theme;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    Ron,
    Toml,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("paths error: {0}")]
    Paths(#[from] paths::Error),

    #[error("ron deserialization error: {0}")]
    Ron(#[from] SpannedError),

    #[error("toml deserialization error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}
