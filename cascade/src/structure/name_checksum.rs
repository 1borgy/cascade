use std::{
    fmt::{self, Debug},
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    lookup,
    structure::{StructureError, Type},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum NameChecksum {
    None,
    Checksum(u32),
    Compressed8(u8),
    Compressed16(u16),
}

impl NameChecksum {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        symbol_type: Type,
        use_lookup_8: bool,
        use_lookup_16: bool,
    ) -> Result<NameChecksum, StructureError> {
        Ok(if symbol_type == Type::None {
            NameChecksum::None
        } else if use_lookup_8 {
            NameChecksum::Compressed8(reader.read_u8()?)
        } else if use_lookup_16 {
            NameChecksum::Compressed16(reader.read_u16::<LittleEndian>()?)
        } else {
            NameChecksum::Checksum(reader.read_u32::<LittleEndian>()?)
        })
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StructureError> {
        match self {
            NameChecksum::Checksum(val) => {
                writer.write_u32::<LittleEndian>(*val)?
            }
            NameChecksum::Compressed8(val) => writer.write_u8(*val)?,
            NameChecksum::Compressed16(val) => {
                writer.write_u16::<LittleEndian>(*val)?
            }
            NameChecksum::None => (),
        }
        Ok(())
    }

    pub fn to_name(&self) -> Option<String> {
        match self {
            NameChecksum::Checksum(checksum) => lookup::checksum(*checksum),
            NameChecksum::Compressed8(byte) => lookup::compressed8(*byte),
            NameChecksum::Compressed16(word) => lookup::compressed16(*word),
            NameChecksum::None => None,
        }
    }
}

impl Serialize for NameChecksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: do some serialize/deserialize shenanigans
        serializer.serialize_str(
            self.to_name().unwrap_or(format!("{:?}", self)).as_str(),
        )
    }
}

impl fmt::Display for NameChecksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.to_name().unwrap_or("<unknown>".to_string());

        let checksum = match self {
            NameChecksum::Checksum(checksum) => {
                format!("Checksum({})", checksum)
            }
            NameChecksum::Compressed8(compressed8) => {
                format!("Compressed8({})", compressed8)
            }
            NameChecksum::Compressed16(compressed16) => {
                format!("Compressed16({})", compressed16)
            }
            NameChecksum::None => format!("None"),
        };

        write!(f, "{} (\"{}\")", checksum, name)
    }
}

impl TryFrom<&String> for NameChecksum {
    type Error = StructureError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        // TODO: can probably micro-optimize depending on which of these is most likely
        if let Some(checksum) = lookup::reverse_checksum(value) {
            Ok(NameChecksum::Checksum(checksum))
        } else if let Some(compressed8) = lookup::reverse_compressed8(value) {
            Ok(NameChecksum::Compressed8(compressed8))
        } else if let Some(compressed16) = lookup::reverse_compressed16(value) {
            Ok(NameChecksum::Compressed16(compressed16))
        } else {
            Err(StructureError::UnknownChecksumName(value.clone()))
        }
    }
}

impl TryFrom<&str> for NameChecksum {
    type Error = StructureError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NameChecksum::try_from(&value.to_string())
    }
}
