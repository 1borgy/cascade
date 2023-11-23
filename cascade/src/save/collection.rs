use std::path::{Path, PathBuf};

use rayon::prelude::*;

use super::extension::SaveFileExtension;
use crate::save::{SaveError, SaveFile};

pub struct SaveCollection {
    saves: Vec<SaveFile>,
}

// TODO: lol this is just a vec how do i properly implement a collection type
impl SaveCollection {
    pub fn at_dir<P: AsRef<Path>>(dir: P) -> Result<Self, SaveError> {
        let dir = PathBuf::from(dir.as_ref());

        dir.is_dir()
            .then(|| ())
            .ok_or_else(|| SaveError::NoSuchDirectory(dir.clone()))?;

        log::info!("using saves from {:?}", dir);

        let saves: Vec<SaveFile> = dir
            .read_dir()?
            .filter_map(|file| file.ok())
            .filter_map(|file| {
                let filepath = file.path();

                match SaveFile::at_path(&filepath) {
                    Ok(save) => {
                        log::info!("found save {:?}", filepath);
                        Some(save)
                    }
                    Err(e) => {
                        log::warn!("error finding save {:?}: {}", filepath, e);
                        None
                    }
                }
            })
            .collect();

        Ok(Self { saves })
    }

    pub fn filter_extension(&self, extension: SaveFileExtension) -> Self {
        Self {
            saves: self
                .saves
                .iter()
                .filter(|save| save.extension() == extension)
                .cloned()
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.saves.len()
    }

    pub fn into_par_iter(self) -> rayon::vec::IntoIter<SaveFile> {
        self.saves.into_par_iter()
    }

    pub fn par_iter<'a>(&'a self) -> rayon::slice::Iter<'a, SaveFile> {
        self.saves.par_iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = SaveFile> {
        self.saves.into_iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SaveFile> {
        self.saves.iter()
    }
}

impl FromIterator<SaveFile> for SaveCollection {
    fn from_iter<T: IntoIterator<Item = SaveFile>>(iter: T) -> Self {
        Self {
            saves: iter.into_iter().collect(),
        }
    }
}
