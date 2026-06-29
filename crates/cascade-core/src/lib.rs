mod cas;
mod entry;
mod save;

use std::{fmt::Display, path::Path};

pub use cas::{Cas, Flags};
pub use entry::Entry;
pub use save::Save;

pub trait Core {
    type Entry: entry::Entry;
    type Save: save::Save;
    type Cas: cas::Cas<Save = Self::Save>;
    type Error: Display;

    fn entries(
        &self,
        dir: impl AsRef<Path>,
    ) -> Result<impl Iterator<Item = Self::Entry>, Self::Error>;
}
