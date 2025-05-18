use std::path::{Path, PathBuf};

mod entry;
mod extension;

pub use entry::Entry;
pub use extension::Extension;

pub use crate::{Error, Result};

pub fn find_entries(dir: impl AsRef<Path>) -> Result<Vec<Entry>> {
    let dir = PathBuf::from(dir.as_ref());

    dir.is_dir()
        .then(|| ())
        .ok_or_else(|| Error::NoSuchDirectory(dir.clone()))?;

    log::info!("finding entries in {:?}", dir);

    Ok(dir
        .read_dir()?
        .filter_map(|file| file.ok())
        .filter_map(|file| {
            let filepath = file.path();

            match Entry::at_path(&filepath) {
                Ok(save) => {
                    log::info!("found entry {:?}", filepath);
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
