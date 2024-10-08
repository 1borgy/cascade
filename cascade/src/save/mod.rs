use std::path::{Path, PathBuf};

mod content;
mod entry;
mod error;
mod extension;
pub mod thug_pro;

pub use content::{Content, Header};
pub use entry::Entry;
pub use error::{Error, Result};
pub use extension::Extension;

pub fn load_entries(dir: impl AsRef<Path>) -> Result<Vec<Entry>> {
    let dir = PathBuf::from(dir.as_ref());

    dir.is_dir()
        .then(|| ())
        .ok_or_else(|| Error::NoSuchDirectory(dir.clone()))?;

    log::info!("loading entries in {:?}", dir);

    Ok(dir
        .read_dir()?
        .filter_map(|file| file.ok())
        .filter_map(|file| {
            let filepath = file.path();

            match Entry::at_path(&filepath) {
                Ok(save) => {
                    log::info!("load entry {:?}", filepath);
                    Some(save)
                }
                Err(e) => {
                    log::warn!("error loading entry {:?}: {}", filepath, e);
                    None
                }
            }
        })
        .collect())
}
