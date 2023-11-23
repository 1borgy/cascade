use std::{backtrace::Backtrace, fmt::Debug};

use cascade_gui::error::CascadeGuiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CascadeCliError {
    #[error("a gui error occurred: {source}")]
    Gui {
        #[from]
        source: CascadeGuiError,
        backtrace: Backtrace,
    },
}
