use std::{backtrace::Backtrace, fmt::Debug};

use thiserror::Error;

use crate::structure::{self, StructureError};

#[derive(Error, Debug)]
pub enum MutationError {
    #[error("a structure operation failed: {source}")]
    Structure {
        #[from]
        source: StructureError,
        backtrace: Backtrace,
    },

    #[error("symbol with checksum {0} could not be found")]
    SymbolNotFound(structure::NameChecksum),

    #[error("mutation spec is invalid; cannot start with leaf")]
    SpecRootIsLeaf,

    #[error("mutation spec is invalid; root must be at top")]
    SpecNodeIsRoot,
}
