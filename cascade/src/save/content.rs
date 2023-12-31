use std::{
    io::{BufRead, Write},
    mem::size_of,
    sync::Arc,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use count_write::CountWrite;
use serde::{Deserialize, Serialize};

use super::SaveError;
use crate::{crc32::get_checksum_for_bytes, structure::Structure};

const SAVE_FILE_SIZE: usize = 90112;
const PADDING_BYTE: u8 = 0x69;

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    pub checksum: u32,
    pub summary_checksum: u32,
    pub summary_size: i32,
    pub total_size: i32,
    pub version: i32,
}

impl Header {
    pub fn from_reader<R: BufRead>(stream: &mut R) -> Result<Self, SaveError> {
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
            summary_checksum: 0,
            summary_size: 0,
            total_size: 0,
            version: self.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveContent {
    // Header is invalid as soon as we make a mutation
    // TODO: remove, just keep version
    header: Header,

    pub summary: Arc<Structure>,
    pub data: Arc<Structure>,
}

impl SaveContent {
    pub fn from_reader<R: BufRead>(reader: &mut R) -> Result<Self, SaveError> {
        Ok(SaveContent {
            header: Header::from_reader(reader)?,
            summary: Arc::new(Structure::from_reader(reader)?),
            data: Arc::new(Structure::from_reader(reader)?),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        let mut count_writer = CountWrite::from(writer);

        // TODO fix this
        let header = self.calculate_header()?;
        header.write(&mut count_writer)?;

        self.summary.write(&mut count_writer)?;
        self.data.write(&mut count_writer)?;

        let num_bytes_written = count_writer.count() as usize;

        let num_padding_bytes =
            SAVE_FILE_SIZE.saturating_sub(num_bytes_written);

        log::info!(
            "wrote {} bytes, padding {} bytes to hit {}",
            num_bytes_written,
            num_padding_bytes,
            SAVE_FILE_SIZE
        );

        count_writer.write(&vec![PADDING_BYTE; num_padding_bytes])?;

        Ok(())
    }

    fn calculate_header(&self) -> Result<Header, SaveError> {
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

        Ok(Header {
            checksum,
            summary_checksum,
            summary_size,
            total_size,
            version: self.header.version,
        })
    }

    pub fn summary(&self) -> Arc<Structure> {
        Arc::clone(&self.summary)
    }

    pub fn data(&self) -> Arc<Structure> {
        Arc::clone(&self.data)
    }

    pub fn with_summary(&self, summary: Arc<Structure>) -> Self {
        Self {
            header: self.header.invalidate(),
            summary,
            data: Arc::clone(&self.data),
        }
    }

    pub fn with_data(&self, data: Arc<Structure>) -> Self {
        Self {
            header: self.header.invalidate(),
            summary: Arc::clone(&self.summary),
            data,
        }
    }
}
