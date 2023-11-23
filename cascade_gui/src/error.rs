use std::{backtrace::Backtrace, fmt::Debug, io};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CascadeGuiError {
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error("an logging error occurred: {source}")]
    Log {
        #[from]
        source: log::SetLoggerError,
        backtrace: Backtrace,
    },

    #[error("a gui error occurred: {source}")]
    Gui {
        #[from]
        source: iced::Error,
        backtrace: Backtrace,
    },
}
