use std::{
    fmt,
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cascade_qb as qb;

pub const MAGIC_DATA: u32 = 0x39137FE5;
pub const MAGIC_SUMMARY: u32 = 0x31D7999C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Magic {
    Data, // aka "MEMCARDSTUFF"
    Summary,
}

impl fmt::Display for Magic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u32> for Magic {
    type Error = cascade_core::Error;

    fn try_from(value: u32) -> cascade_core::Result<Self> {
        match value {
            MAGIC_DATA => Ok(Magic::Data),
            MAGIC_SUMMARY => Ok(Magic::Summary),
            _ => Err(cascade_core::Error::UnknownChunkMagic(value)),
        }
    }
}
impl From<Magic> for u32 {
    fn from(val: Magic) -> Self {
        match val {
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
    pub structure: qb::Structure,
}

impl Chunk {
    pub fn read(reader: &mut impl Read) -> cascade_core::Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?.try_into()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let structure = qb::Structure::read(reader)?;
        // TODO: use size?
        // let mut data = vec![0; size as usize];
        // reader.read_exact(&mut data)?;

        Ok(Self {
            magic,
            size,
            structure,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> cascade_core::Result<()> {
        let chunk_size = self.structure.raw_bytes()?.len();
        writer.write_u32::<LittleEndian>(self.magic.into())?;
        writer.write_u32::<LittleEndian>(chunk_size as u32)?;
        self.structure.write(writer)?;
        Ok(())
    }
}
