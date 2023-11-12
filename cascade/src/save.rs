use std::{
    backtrace::Backtrace,
    io::{self, Read, Write},
    mem::size_of,
    rc::Rc,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use count_write::CountWrite;
use serde::{Deserialize, Serialize};
use symbol::Structure;
use thiserror::Error;

use crate::{
    crc32::get_checksum_for_bytes,
    symbol::{self, SymbolError},
};

const SAVE_FILE_SIZE: usize = 90112;
const PADDING_BYTE: u8 = 0x69;

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
    #[error("an error occurred while reading/writing symbols")]
    Symbol {
        #[from]
        source: SymbolError,
        backtrace: Backtrace,
    },
    #[error("the name field was observed to be invalid, got: {0}")]
    InvalidName(symbol::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub checksum: u32,
    pub summary_checksum: u32,
    pub summary_size: i32,
    pub total_size: i32,
    pub version: i32,
}

impl Header {
    pub fn from_reader<R: Read>(stream: &mut R) -> Result<Self, SaveError> {
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

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        writer.write_u32::<LittleEndian>(self.checksum)?;
        writer.write_u32::<LittleEndian>(self.summary_checksum)?;
        writer.write_i32::<LittleEndian>(self.summary_size)?;
        writer.write_i32::<LittleEndian>(self.total_size)?;
        writer.write_i32::<LittleEndian>(self.version)?;
        Ok(())
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>, SaveError> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;

        Ok(bytes)
    }

    pub fn invalidate(&self) -> Header {
        Header {
            checksum: 0,
            summary_checksum: self.summary_checksum,
            summary_size: self.summary_size,
            total_size: 0,
            version: self.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    header: Header,
    summary: Rc<Structure>,
    data: Rc<Structure>,
}

impl SaveData {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, SaveError> {
        Ok(SaveData {
            header: Header::from_reader(reader)?,
            summary: Rc::new(Structure::from_reader(reader)?),
            data: Rc::new(Structure::from_reader(reader)?),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        let mut count_writer = CountWrite::from(writer);

        self.header.write(&mut count_writer)?;
        self.summary.write(&mut count_writer)?;
        self.data.write(&mut count_writer)?;

        let num_bytes_written = count_writer.count() as usize;
        log::debug!("save file len before padding is {}", num_bytes_written);

        for _ in 0..(SAVE_FILE_SIZE - num_bytes_written) {
            count_writer.write_u8(PADDING_BYTE)?;
        }

        Ok(())
    }

    pub fn with_recalculated_header(&self) -> Result<SaveData, SaveError> {
        let mut summary_bytes = self.summary.raw_bytes()?;
        let mut data_bytes = self.data.raw_bytes()?;

        let summary_checksum = get_checksum_for_bytes(&summary_bytes);
        let summary_size = summary_bytes.len() as i32;
        let data_size = data_bytes.len() as i32;
        let total_size = size_of::<Header>() as i32 + summary_size + data_size;

        let header_zero_checksum = Header {
            checksum: 0,
            summary_checksum,
            summary_size,
            total_size,
            version: self.header.version,
        };

        let mut all_bytes = vec![];
        all_bytes.append(&mut header_zero_checksum.raw_bytes()?);
        all_bytes.append(&mut summary_bytes);
        all_bytes.append(&mut data_bytes);

        let checksum = get_checksum_for_bytes(&all_bytes);

        Ok(SaveData {
            header: Header {
                checksum,
                summary_checksum,
                summary_size,
                total_size,
                version: self.header.version,
            },
            summary: self.summary.clone(),
            data: self.data.clone(),
        })
    }

    pub fn name(&self) -> Result<String, SaveError> {
        let symbol = self.summary.try_get("Filename")?;

        match &symbol.value {
            symbol::Value::String(str) => Ok(str.clone()),
            other => Err(SaveError::InvalidName(other.clone())),
        }
    }

    pub fn with_name(&self, name: &str) -> Result<SaveData, SaveError> {
        let cas_filename_path =
            ["CustomSkater", "custom", "info", "CASFileName"];

        Ok(SaveData {
            header: self.header.invalidate(),
            summary: self.summary.with_replaced_symbol(
                "Filename",
                Rc::new(
                    self.summary
                        .try_get("Filename")?
                        .with_value(symbol::Value::String(name.to_string())),
                ),
            )?,
            data: self.data.with_copied_path(&self.data, &cas_filename_path)?,
        })
    }

    pub fn with_copied_trickset(
        &self,
        other: &SaveData,
    ) -> Result<SaveData, SaveError> {
        let trick_binding_path =
            ["CustomSkater", "custom", "info", "trick_mapping"];
        let cat_path = ["StorySkater", "tricks"];
        let specials_path = ["CustomSkater", "custom", "info", "specials"];

        Ok(SaveData {
            header: self.header.invalidate(),
            summary: Rc::clone(&self.summary),
            // idk this is pretty ugly like do i really need to make 3 different copies
            // compiler optimizations please save me
            data: self
                .data
                .with_copied_path(&other.data, &trick_binding_path)?
                .with_copied_path(&other.data, &cat_path)?
                .with_copied_path(&other.data, &specials_path)?,
        })
    }
}
