use std::path::Path;

use crate::{Cas, Entry, Error, Result, Save, entry::find_entries};

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
