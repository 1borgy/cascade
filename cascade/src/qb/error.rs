use std::{fmt::Debug, io};

use crate::qb::Value;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("invalid symbol type {0}")]
    InvalidType(u8),

    #[error("internal error: {0} is not implemented")]
    NotImplemented(String),

    #[error(
        "both checksum table lookup bits set in symbol type byte: {0:#02x}"
    )]
    BothChecksumBits(u8),

    #[error("expected value type {0}, got {1}")]
    ExpectedValueType(String, Value),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}
