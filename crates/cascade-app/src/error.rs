use std::{io, result};

use ron::de::SpannedError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("ron deserialization error: {0}")]
    RonDe(#[from] SpannedError),

    #[error("ron serialization error: {0}")]
    RonSer(#[from] ron::Error),

    #[error("toml deserialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("core error: {0}")]
    Core(#[from] cascade_core::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;
