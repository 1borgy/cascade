use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cascade_save as save;

use crate::{
    Error, Result,
    chunk::{self, Chunk},
};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rethawed {
    pub magic: u32,
    pub version: u32,
    pub chunk_count: u32,
    pub chunks: Vec<Chunk>,
}

impl cascade_core::Save for Rethawed {
    type Error = Error;

    fn read(reader: &mut (impl Read + Seek)) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;
        let version = reader.read_u32::<LittleEndian>()?;
        let chunk_count = reader.read_u32::<LittleEndian>()?;

        let mut chunks = Vec::with_capacity(chunk_count as usize);
        for _ in 0..chunk_count {
            chunks.push(Chunk::read(reader)?);
        }

        Ok(Self {
            magic,
            version,
            chunk_count,
            chunks,
        })
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.magic)?;
        writer.write_u32::<LittleEndian>(self.version)?;
        writer.write_u32::<LittleEndian>(self.chunk_count)?;

        for chunk in self.chunks.iter() {
            chunk.write(writer)?;
        }

        Ok(())
    }
}

impl Rethawed {
    // the following methods assume each type of chunk only exists once per save

    pub fn get_chunk(&self, magic: chunk::Magic) -> Option<&Chunk> {
        self.chunks.iter().find(|chunk| chunk.magic == magic)
    }

    pub fn try_get_chunk(&self, magic: chunk::Magic) -> Result<&Chunk> {
        self.get_chunk(magic)
            .ok_or_else(|| Error::ChunkNotFound(magic))
    }

    pub fn get_chunk_mut(&mut self, magic: chunk::Magic) -> Option<&mut Chunk> {
        self.chunks.iter_mut().find(|chunk| chunk.magic == magic)
    }

    pub fn try_get_chunk_mut(&mut self, magic: chunk::Magic) -> Result<&mut Chunk> {
        self.get_chunk_mut(magic)
            .ok_or_else(|| Error::ChunkNotFound(magic))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Save {
    Thaw(save::Save),
    Rethawed(Rethawed),
}

impl cascade_core::Save for Save {
    type Error = Error;

    fn read(reader: &mut (impl Read + Seek)) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;
        reader.seek(SeekFrom::Start(0))?;

        match magic {
            // "RTHW"
            0x57485452 => Ok(Save::Rethawed(Rethawed::read(reader)?)),
            _ => Ok(Save::Thaw(save::Save::read(reader)?)),
        }
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        match self {
            Save::Thaw(save) => Ok(save.write(writer)?),
            Save::Rethawed(save) => Ok(save.write(writer)?),
        }
    }
}
