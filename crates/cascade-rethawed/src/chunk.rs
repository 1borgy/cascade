use std::{
    fmt,
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cascade_qb as qb;

use crate::{Error, Result};

pub const MAGIC_DATA: u32 = 0x39137FE5;
pub const MAGIC_SUMMARY: u32 = 0x31D7999C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Magic {
    Data, // "MEMCARDSTUFF"
    Summary,
}

impl fmt::Display for Magic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u32> for Magic {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            MAGIC_DATA => Ok(Magic::Data),
            MAGIC_SUMMARY => Ok(Magic::Summary),
            _ => Err(Error::UnknownChunkMagic(value)),
        }
    }
}
impl Into<u32> for Magic {
    fn into(self) -> u32 {
        match self {
            Magic::Data => MAGIC_DATA,
            Magic::Summary => MAGIC_SUMMARY,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chunk {
    pub magic: Magic,
    pub size: u32,
    pub structure: Box<qb::Structure>,
}

impl Chunk {
    pub fn read(reader: &mut impl Read) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?.try_into()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let structure = Box::new(qb::Structure::read(reader)?);
        // TODO: use size?
        // let mut data = vec![0; size as usize];
        // reader.read_exact(&mut data)?;

        Ok(Self {
            magic,
            size,
            structure,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<()> {
        let chunk_size = self.structure.raw_bytes()?.len();
        writer.write_u32::<LittleEndian>(self.magic.into())?;
        writer.write_u32::<LittleEndian>(chunk_size as u32)?;
        self.structure.write(writer)?;
        Ok(())
    }
}
