use std::{
    fs::{self},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use filetime;

use crate::save::{extension::Extension, Content, Error, Result};

#[derive(Debug, Clone)]
pub struct Entry {
    pub dir: PathBuf,
    pub name: String,
    pub extension: Extension,
    metadata: fs::Metadata,
}

impl Entry {
    pub fn at_path<P: AsRef<Path>>(filepath: P) -> Result<Self> {
        let metadata = fs::metadata(&filepath)?;

        let extension = Extension::try_from(
            filepath
                .as_ref()
                .extension()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref()))
                })?,
        )?;

        let name = filepath
            .as_ref()
            .with_extension("")
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| {
                Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref()))
            })?;

        let dir = filepath
            .as_ref()
            .parent()
            .map(|dir| PathBuf::from(dir))
            .ok_or_else(|| {
                Error::InvalidSaveFilePath(PathBuf::from(filepath.as_ref()))
            })?;

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

    //pub fn metadata(&self) -> &fs::Metadata {
    //    &self.metadata
    //}

    pub fn write_content(&self, content: &Content) -> Result<()> {
        let filepath = self.filepath();
        let file = fs::File::create(&filepath)?;
        let mut writer = BufWriter::new(file);

        // TODO: should we set the filename in the content structures?
        content.write(&mut writer)?;
        writer.flush()?;

        log::info!("wrote save to {:?}", filepath);

        Ok(())
    }

    pub fn overwrite_metadata(&self) -> Result<()> {
        let filepath = self.filepath();

        // TODO: this should probably be configurable
        let original_mod_time =
            filetime::FileTime::from_last_modification_time(&self.metadata);

        // TODO: how tf do i format this
        log::info!(
            "setting file modification time for {:?} to {:?}",
            filepath,
            original_mod_time
        );
        filetime::set_file_mtime(&filepath, original_mod_time)?;

        Ok(())
    }

    pub fn load_content(&self) -> Result<Content> {
        let filepath = self.filepath();

        let file = fs::File::open(&filepath)?;
        let mut reader = BufReader::new(file);

        Content::read(&mut reader)
    }
}
