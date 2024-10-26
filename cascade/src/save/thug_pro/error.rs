use std::{io, result};

use crate::{qb, save};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("qb error: {0}")]
    Qb(#[from] qb::Error),

    #[error("save error: {0}")]
    Save(#[from] save::Error),

    #[error("symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("symbol not found: {0}")]
    ExpectedStructure(String, qb::Value),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;
