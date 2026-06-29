use std::path::Path;

use crate::{
    Error, Result,
    cas::Cas,
    entry::{Entry, find_entries},
    save::Save,
};

pub struct Core {}

impl cascade_core::Core for Core {
    type Entry = Entry;
    type Save = Save;
    type Cas = Cas;
    type Error = Error;

    fn entries(&self, dir: impl AsRef<Path>) -> Result<impl Iterator<Item = Entry>> {
        find_entries(dir)
    }
}
