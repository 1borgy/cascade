use std::{backtrace::Backtrace, io, path::PathBuf};

use thiserror::Error;

use crate::structure::StructureError;

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error("an error occurred while reading/writing symbols")]
    Symbol {
        #[from]
        source: StructureError,
        backtrace: Backtrace,
    },

    #[error("unknown save file extension {0}")]
    UnknownFileExtension(String),

    #[error("directory \"{0}\" was not found")]
    NoSuchDirectory(PathBuf),

    #[error("save file path \"{0}\" is not valid")]
    InvalidSaveFilePath(PathBuf),
}
