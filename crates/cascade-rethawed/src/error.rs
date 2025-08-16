use std::{io, path::PathBuf, result};

use cascade_qb as qb;
use cascade_save as save;

use crate::chunk;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("qb error: {0}")]
    Qb(#[from] qb::Error),

    #[error("save error: {0}")]
    Save(#[from] save::Error),

    #[error("unknown chunk magic: {0}")]
    UnknownChunkMagic(u32),

    #[error("symbol not found: {0}")]
    SymbolNotFound(qb::Id),

    #[error("chunk not found: {0}")]
    ChunkNotFound(chunk::Magic),

    #[error("no such directory: \"{0}\"")]
    NoSuchDirectory(PathBuf),

    #[error("invalid save file path: \"{0}\"")]
    InvalidSaveFilePath(PathBuf),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;
