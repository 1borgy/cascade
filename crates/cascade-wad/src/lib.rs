use std::{io, result, str::Utf8Error};
pub mod hed;
pub mod wad;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("utf8 error: {0}")]
    Utf8(#[from] Utf8Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;
