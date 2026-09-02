mod cas;
mod id;
#[cfg(feature = "dump")]
pub mod lut;
mod save;

pub use cas::Cas;
pub use save::Save;

pub const FILTER_NAME: &str = "THUG2 CAS";
pub const FILTER_EXTENSION: &str = "SKA";

pub fn find_entries(dir: &std::path::PathBuf) -> Vec<cascade_core::Entry> {
    cascade_core::find_entries(dir, FILTER_EXTENSION)
}
