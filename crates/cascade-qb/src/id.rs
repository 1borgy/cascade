use std::{
    fmt::{Debug, Display},
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{Error, Kind};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Id {
    None,
    Checksum(u32),
    Compress8(u8),
    Compress16(u16),
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Id {
    pub fn read(
        reader: &mut impl Read,
        kind: Kind,
        compressed_8: bool,
        compressed_16: bool,
    ) -> Result<Id, Error> {
        Ok(if kind == Kind::None {
            Id::None
        } else if compressed_8 {
            Id::Compress8(reader.read_u8()?)
        } else if compressed_16 {
            Id::Compress16(reader.read_u16::<LittleEndian>()?)
        } else {
            Id::Checksum(reader.read_u32::<LittleEndian>()?)
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Id::Checksum(val) => writer.write_u32::<LittleEndian>(*val)?,
            Id::Compress8(val) => writer.write_u8(*val)?,
            Id::Compress16(val) => writer.write_u16::<LittleEndian>(*val)?,
            Id::None => (),
        }
        Ok(())
    }
}
