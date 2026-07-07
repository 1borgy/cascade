mod cas;
mod chunk;
#[cfg(feature = "lut")]
pub mod dump;
mod id;
#[cfg(feature = "lut")]
pub mod lut;
mod save;

pub use cas::Cas;
pub use save::Save;

pub const FILTER_NAME: &'static str = "THAW CAS file (-Progress)";
pub const FILTER_EXTENSION: &'static str = "-Progress";

pub fn find_entries(dir: &std::path::PathBuf) -> Vec<cascade_core::Entry> {
    cascade_core::find_entries(dir, FILTER_EXTENSION)
}
