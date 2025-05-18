use std::{
    fmt,
    fs::{self},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cascade_crc as crc;
use cascade_qb as qb;
use count_write::CountWrite;

use crate::{Error, Result};

const SAVE_FILE_SIZE: usize = 90112;
const PADDING_BYTE: u8 = 0x69;

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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub checksum: u32,
    pub summary_checksum: u32,
    pub summary_size: i32,
    pub total_size: i32,
    pub version: i32,
}

impl Header {
    pub fn read(stream: &mut impl Read) -> Result<Self> {
        let checksum = stream.read_u32::<LittleEndian>()?;
        let summary_checksum = stream.read_u32::<LittleEndian>()?;
        let summary_size = stream.read_i32::<LittleEndian>()?;
        let data_size = stream.read_i32::<LittleEndian>()?;
        let version = stream.read_i32::<LittleEndian>()?;

        Ok(Header {
            checksum,
            summary_checksum,
            summary_size,
            total_size: data_size,
            version,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.checksum)?;
        writer.write_u32::<LittleEndian>(self.summary_checksum)?;
        writer.write_i32::<LittleEndian>(self.summary_size)?;
        writer.write_i32::<LittleEndian>(self.total_size)?;
        writer.write_i32::<LittleEndian>(self.version)?;
        Ok(())
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;

        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct Save {
    pub header: Header,

    pub summary: Box<qb::Structure>,
    pub data: Box<qb::Structure>,
}

impl Save {
    pub fn read(reader: &mut impl Read) -> Result<Self> {
        Ok(Self {
            header: Header::read(reader)?,
            summary: Box::new(qb::Structure::read(reader)?),
            data: Box::new(qb::Structure::read(reader)?),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut count_writer = CountWrite::from(writer);

        // TODO fix this
        let header = self.calculate_header()?;
        header.write(&mut count_writer)?;

        self.summary.write(&mut count_writer)?;
        self.data.write(&mut count_writer)?;

        let num_bytes_written = count_writer.count() as usize;

        let num_padding_bytes = SAVE_FILE_SIZE.saturating_sub(num_bytes_written);

        log::info!(
            "wrote {} bytes, padding with {} bytes to fill {} bytes",
            num_bytes_written,
            num_padding_bytes,
            SAVE_FILE_SIZE
        );

        count_writer.write(&vec![PADDING_BYTE; num_padding_bytes])?;

        Ok(())
    }

    pub fn read_from(entry: &Entry) -> Result<Self> {
        let mut reader = entry.reader()?;
        Self::read(&mut reader)
    }

    pub fn write_to(self, entry: &Entry) -> Result<()> {
        let mut reader = entry.writer()?;
        self.write(&mut reader)
    }

    fn calculate_header(&self) -> Result<Header> {
        let mut summary_bytes = self.summary.raw_bytes()?;
        let mut data_bytes = self.data.raw_bytes()?;

        let summary_checksum = crc::checksum(&summary_bytes);
        let summary_size = summary_bytes.len() as i32;
        let data_size = data_bytes.len() as i32;
        let total_size = size_of::<Header>() as i32 + summary_size + data_size;
        let version = self.header.version;

        let header_zero_checksum = Header {
            checksum: 0,
            summary_checksum,
            summary_size,
            total_size,
            version,
        };

        let mut all_bytes = vec![];
        all_bytes.append(&mut header_zero_checksum.raw_bytes()?);
        all_bytes.append(&mut summary_bytes);
        all_bytes.append(&mut data_bytes);

        let checksum = crc::checksum(&all_bytes);

        Ok(Header {
            checksum,
            summary_checksum,
            summary_size,
            total_size,
            version,
        })
    }

    pub fn with_summary(&self, summary: Box<qb::Structure>) -> Self {
        Self {
            header: self.header.clone(),
            summary,
            data: Box::clone(&self.data),
        }
    }

    pub fn with_data(&self, data: Box<qb::Structure>) -> Self {
        Self {
            header: self.header.clone(),
            summary: Box::clone(&self.summary),
            data,
        }
    }
}
