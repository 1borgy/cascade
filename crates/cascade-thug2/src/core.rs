use std::path::PathBuf;

use cascade_save::Save;

use crate::{
    Error, Result,
    cas::Cas,
    entry::{Entry, find_entries},
};

pub struct Core {
    cwd: PathBuf,
}

impl Core {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

impl cascade_core::Core for Core {
    type Entry = Entry;
    type Save = Save;
    type Cas = Cas;
    type Error = Error;

    fn list_entries(&self) -> Result<impl Iterator<Item = Entry>> {
        find_entries(&self.cwd)
    }
}
