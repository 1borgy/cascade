use std::{
    fmt::Debug,
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::qb::{Error, Kind};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Id {
    None,
    Checksum(u32),
    Compressed8(u8),
    Compressed16(u16),
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
            Id::Compressed8(reader.read_u8()?)
        } else if compressed_16 {
            Id::Compressed16(reader.read_u16::<LittleEndian>()?)
        } else {
            Id::Checksum(reader.read_u32::<LittleEndian>()?)
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Id::Checksum(val) => writer.write_u32::<LittleEndian>(*val)?,
            Id::Compressed8(val) => writer.write_u8(*val)?,
            Id::Compressed16(val) => writer.write_u16::<LittleEndian>(*val)?,
            Id::None => (),
        }
        Ok(())
    }
}
