use std::{io, path::PathBuf, result};

use crate::qb;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("structure error: {0}")]
    Symbol(#[from] qb::Error),

    #[error("unknown save file extension \"{0}\"")]
    UnknownFileExtension(String),

    #[error("directory \"{0}\" was not found")]
    NoSuchDirectory(PathBuf),

    #[error("save file path \"{0}\" is not valid")]
    InvalidSaveFilePath(PathBuf),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;
