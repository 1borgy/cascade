use std::path::PathBuf;

use cascade_backend as backend;
use cascade_save as save;

use crate::{Cas, Entry, Error, entry::find_entries};

pub struct Backend {
    saves_dir: PathBuf,
}

impl backend::Target for Backend {
    type Entry = Entry;
    type Error = Error;

    fn list(&self) -> Result<impl IntoIterator<Item = Self::Entry>, Self::Error> {
        find_entries(&self.saves_dir)
    }

    fn name(&self, entry: &Self::Entry) -> impl AsRef<str> {
        &entry.name
    }

    fn read(&self, entry: &Self::Entry) -> Result<save::Save, Self::Error> {
        Ok(save::Save::read(&mut entry.reader()?)?)
    }

    fn write(&self, entry: &Self::Entry, save: &save::Save) -> Result<(), Self::Error> {
        Ok(save.write(&mut entry.writer()?)?)
    }
}

impl backend::Structure for Backend {
    type Parsed = Cas;
    type Error = Error;

    fn parse(&self, save: &save::Save) -> Result<Self::Parsed, Self::Error> {
        Ok(Cas::try_from(save)?)
    }

    fn modify(&self, save: &mut save::Save, transform: Self::Parsed) -> Result<(), Self::Error> {
        Ok(transform.modify(save)?)
    }
}
