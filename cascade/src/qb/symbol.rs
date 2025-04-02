use std::{
    fmt::Debug,
    io::{Read, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::Structure;
use crate::qb::{Error, Id, Kind, Value};

const CHECKSUM_LOOKUP_MASK_8: u8 = 1 << 7;
const CHECKSUM_LOOKUP_MASK_16: u8 = 1 << 6;

#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
    pub kind: Kind,
    pub id: Id,
    pub value: Value,
}

impl Symbol {
    pub fn none() -> Self {
        Self {
            kind: Kind::None,
            id: Id::None,
            value: Value::None,
        }
    }

    pub fn structure(id: Id, structure: Box<Structure>) -> Self {
        Self {
            kind: Kind::Structure,
            id,
            value: Value::Structure(structure),
        }
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let type_byte = reader.read_u8()?;

        // 8-bit / 16-bit mask in bits 6/7
        let type_byte_masked =
            type_byte & (!CHECKSUM_LOOKUP_MASK_8) & (!CHECKSUM_LOOKUP_MASK_16);

        let use_lookup_8 = (type_byte & CHECKSUM_LOOKUP_MASK_8) > 0;
        let use_lookup_16 = (type_byte & CHECKSUM_LOOKUP_MASK_16) > 0;

        // Error if both checksum bits are set
        (!(use_lookup_8 && use_lookup_16))
            .then(|| ())
            .ok_or(Error::BothChecksumBits(type_byte))?;

        let kind = Kind::try_from(type_byte_masked)?;

        let id = Id::read(reader, kind, use_lookup_8, use_lookup_16)?;

        let value = Value::read(reader, kind)?;
        let symbol = Symbol { kind, id, value };

        Ok(symbol)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let type_byte = self.kind as u8;

        let type_byte_with_checksum_bits = match self.id {
            Id::Compressed16(_) => type_byte | CHECKSUM_LOOKUP_MASK_16,
            Id::Compressed8(_) => type_byte | CHECKSUM_LOOKUP_MASK_8,
            _ => type_byte,
        };

        writer.write_u8(type_byte_with_checksum_bits)?;
        self.id.write(writer)?;
        self.value.write(writer)?;

        Ok(())
    }
}
