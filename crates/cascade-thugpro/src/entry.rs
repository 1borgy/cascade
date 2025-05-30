use std::{
    fmt,
    fs::{self},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Extension {
    SKA,
}

impl TryFrom<&str> for Extension {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "SKA" => Ok(Extension::SKA),
            _ => Err(Error::UnknownFileExtension(value.to_string())),
        }
    }
}

impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Extension::SKA => "SKA",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub dir: PathBuf,
    pub name: String,
    pub extension: Extension,
    pub metadata: fs::Metadata,
}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filepath().hash(state);
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        // TODO: store filepath in entry
        self.filepath() == other.filepath()
    }
}

impl Eq for Entry {}

impl Entry {
    pub fn at_path<P: AsRef<Path>>(filepath: P) -> Result<Self> {
        let metadata = fs::metadata(&filepath)?;

        let extension = Extension::try_from(
            filepath
                .as_ref()
                .extension()
                .and_then(|name| name.to_str())
                .ok_or_else(|| Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref())))?,
        )?;

        let name = filepath
            .as_ref()
            .with_extension("")
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref())))?;

        let dir = filepath
            .as_ref()
            .parent()
            .map(|dir| PathBuf::from(dir))
            .ok_or_else(|| Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref())))?;

        Ok(Self {
            dir,
            name,
            extension,
            metadata,
        })
    }

    pub fn with_dir<P: AsRef<Path>>(&self, dir: P) -> Self {
        Self {
            dir: PathBuf::from(dir.as_ref()),
            name: self.name.clone(),
            extension: self.extension,
            metadata: self.metadata.clone(),
        }
    }

    pub fn with_name(&self, name: impl ToString) -> Self {
        Self {
            dir: self.dir.clone(),
            name: name.to_string(),
            extension: self.extension,
            metadata: self.metadata.clone(),
        }
    }

    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.extension)
    }

    pub fn filepath(&self) -> PathBuf {
        self.dir.join(self.filename())
    }

    pub fn metadata(&self) -> &fs::Metadata {
        &self.metadata
    }

    pub fn reader(&self) -> Result<impl Read> {
        let file = fs::File::open(&self.filepath())?;
        Ok(BufReader::new(file))
    }

    pub fn writer(&self) -> Result<impl Write> {
        let file = fs::File::create(&self.filepath())?;
        Ok(BufWriter::new(file))
    }

    pub fn overwrite_metadata(&self) -> Result<()> {
        let filepath = self.filepath();

        // TODO: this should probably be configurable
        let original_mod_time = filetime::FileTime::from_last_modification_time(&self.metadata);

        // TODO: how tf do i format this
        log::info!(
            "setting file modification time for {:?} to {:?}",
            filepath,
            original_mod_time
        );
        filetime::set_file_mtime(&filepath, original_mod_time)?;

        Ok(())
    }
}

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
