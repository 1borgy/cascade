mod cas;
mod entry;
mod save;

pub use cas::{Cas, Flags};
pub use entry::Entry;
pub use save::Save;

pub trait Core {
    type Entry: entry::Entry;
    type Save: save::Save;
    type Cas: cas::Cas<Save = Self::Save>;
    type Error;

    fn list_entries(&self) -> Result<impl Iterator<Item = Self::Entry>, Self::Error>;
}
