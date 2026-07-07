use std::{
    fs,
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

    pub fn with_dir<P: AsRef<Path>>(&self, dir: P) -> Self {
        let dir = PathBuf::from(dir.as_ref());
        Self {
            path: dir.join(&self.filename),
            dir: PathBuf::from(dir),
            filename: self.filename.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn reader(&self) -> Result<impl Read + Seek> {
        let file = fs::File::open(&self.path)?;
        Ok(BufReader::new(file))
    }

    pub fn writer(&self) -> Result<impl Write + Seek> {
        let file = fs::File::create(&self.path)?;
        Ok(BufWriter::new(file))
    }

    pub fn name(&self) -> &String {
        &self.filename
    }

    pub fn rewrite_metadata(&self) -> Result<()> {
        if let Some(metadata) = &self.metadata {
            let filepath = &self.path;

            // TODO: this should probably be configurable
            let original_mod_time = filetime::FileTime::from_last_modification_time(metadata);

            // TODO: how tf do i format this
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

pub fn find_entries(dir: &PathBuf, extension: &str) -> Vec<Entry> {
    match dir.read_dir() {
        Ok(dir) => {
            log::debug!("finding entries in {:?}", dir);
            dir.filter_map(|file| file.ok())
                .filter_map(|file| {
                    let filepath = file.path();

                    // oops i should probably make this cleaner
                    if let Some(filename) = filepath.file_name() {
                        if filename.to_string_lossy().ends_with(extension) {
                            match Entry::create(&filepath) {
                                Ok(save) => {
                                    log::debug!("found entry {:?}", filepath);
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
                })
                .collect()
        }
        Err(err) => {
            log::error!("error reading directory {:?}: {}", dir, err);
            Vec::new()
        }
    }
}
