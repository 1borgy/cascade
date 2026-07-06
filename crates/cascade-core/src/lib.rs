mod cas;
mod entry;
mod error;
mod save;

pub use cas::{Cas, Flags};
pub use entry::Entry as Entry;
pub use error::{Error, Result};
pub use save::Save;

pub trait Core {
    type Save: save::Save;
    type Cas: cas::Cas<Save = Self::Save>;
}
