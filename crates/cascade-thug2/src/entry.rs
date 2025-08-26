use std::{
    fs::{self},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
};

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub dir: PathBuf,
    pub filename: String,
    pub metadata: Option<fs::Metadata>,
}

impl cascade_core::Entry for Entry {
    type Error = Error;

    fn reader(&self) -> Result<impl Read + Seek> {
        let file = fs::File::open(&self.path)?;
        Ok(BufReader::new(file))
    }

    fn writer(&self) -> Result<impl Write + Seek> {
        let file = fs::File::create(&self.path)?;
        Ok(BufWriter::new(file))
    }

    fn name(&self) -> String {
        self.filename.clone()
    }

    fn rewrite_metadata(&self) -> Result<()> {
        if let Some(metadata) = &self.metadata {
            let filepath = &self.path;

            let original_mod_time = filetime::FileTime::from_last_modification_time(metadata);

            log::info!(
                "setting file modification time for {:?} to {:?}",
                filepath,
                original_mod_time
            );
            filetime::set_file_mtime(&filepath, original_mod_time)?;
        }

        Ok(())
    }
}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        // TODO: store filepath in entry
        self.path == other.path
    }
}

impl Eq for Entry {}

impl Entry {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let metadata = fs::metadata(path).ok();
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into())
            .ok_or_else(|| Error::InvalidSaveFilePath(PathBuf::from(path)))?;

        let dir = path
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| Error::InvalidSaveFilePath(PathBuf::from(path)))?;

        Ok(Self {
            path: PathBuf::from(path),
            dir,
            filename,
            metadata,
        })
    }

    pub fn reader(&self) -> Result<impl Read + Seek> {
        let file = fs::File::open(&self.path)?;
        Ok(BufReader::new(file))
    }

    pub fn writer(&self) -> Result<impl Write> {
        let file = fs::File::create(&self.path)?;
        Ok(BufWriter::new(file))
    }
}

pub fn find_entries(dir: impl AsRef<Path>) -> Result<impl Iterator<Item = Entry>> {
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
            log::info!("found file at {:?}", filepath);

            // oops i should probably make this cleaner
            if let Some(filename) = filepath.file_name() {
                if filename.to_string_lossy().ends_with(".SKA") {
                    match Entry::create(&filepath) {
                        Ok(save) => {
                            log::info!("found entry {:?}", filepath);
                            Some(save)
                        }
                        Err(e) => {
                            log::warn!("error loading entry {:?}: {}", filepath, e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }))
}
