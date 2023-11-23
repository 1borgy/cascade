use std::{backtrace::Backtrace, fmt::Debug, io};

use thiserror::Error;

use super::NameChecksum;

#[derive(Error, Debug)]
pub enum StructureError {
    #[error("invalid symbol type {0}")]
    InvalidType(u8),

    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error("internal error: {0} is not implemented")]
    NotImplemented(String),

    #[error("symbol with checksum {0} is not a structure")]
    NotAStructure(NameChecksum),

    #[error(
        "both checksum table lookup bits set in symbol type byte: {0:#02x}"
    )]
    BothChecksumBits(u8),

    #[error("could not find name \"{0}\" in lookup tables")]
    UnknownChecksumName(String),

    #[error("could not find symbol with name {0} in structure")]
    NameNotFound(String),

    #[error("could not find symbol with name checksum {0} in structure")]
    NameChecksumNotFound(NameChecksum),

    #[error("index {0} is out of range of the structure")]
    IndexOutOfRange(usize),
}
