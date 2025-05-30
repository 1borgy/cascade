use std::{io, path::PathBuf, result};

use cascade_qb as qb;
use cascade_save as save;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("qb error: {0}")]
    Qb(#[from] qb::Error),

    #[error("save error: {0}")]
    Save(#[from] save::Error),

    #[error("unknown save file extension \"{0}\"")]
    UnknownFileExtension(String),

    #[error("symbol not found: {0}")]
    SymbolNotFound(qb::Id),

    #[error("symbol not found: {0}")]
    ExpectedStructure(String, qb::Value),

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
