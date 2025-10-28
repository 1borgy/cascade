use std::path::PathBuf;

use crate::{Cas, Entry, Error, Result, Save, entry::find_entries};

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

    fn entries(&self) -> Result<impl Iterator<Item = Entry>> {
        find_entries(&self.cwd)
    }
}
