use std::{backtrace::Backtrace, fmt::Debug, io};

use thiserror::Error;

use crate::{mutations::MutationError, save::SaveError};

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("a mutation failed: {source}")]
    Mutation {
        #[from]
        source: MutationError,
        backtrace: Backtrace,
    },

    #[error("a save operation failed: {source}")]
    Save {
        #[from]
        source: SaveError,
        backtrace: Backtrace,
    },

    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
}
