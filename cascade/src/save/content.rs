use std::{
    io::{Read, Write},
    mem::size_of,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use count_write::CountWrite;
use serde::{Deserialize, Serialize};

use crate::{crc32, qb, save::Result};

const SAVE_FILE_SIZE: usize = 90112;
const PADDING_BYTE: u8 = 0x69;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct Content {
    pub header: Header,

    pub summary: Box<qb::Structure>,
    pub data: Box<qb::Structure>,
}

impl Content {
    pub fn read(reader: &mut impl Read) -> Result<Self> {
        Ok(Content {
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

    fn calculate_header(&self) -> Result<Header> {
        let mut summary_bytes = self.summary.raw_bytes()?;
        let mut data_bytes = self.data.raw_bytes()?;

        let summary_checksum = crc32::checksum(&summary_bytes);
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

        let checksum = crc32::checksum(&all_bytes);

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
