use std::{
    fmt::Debug,
    io::{BufRead, Write},
    sync::Arc,
};

use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::structure::{NameChecksum, Structure, StructureError, Type, Value};

const CHECKSUM_LOOKUP_MASK_8: u8 = 1 << 7;
const CHECKSUM_LOOKUP_MASK_16: u8 = 1 << 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    symbol_type: Type,
    name_checksum: NameChecksum,
    value: Value,
}

impl Symbol {
    pub fn none() -> Symbol {
        Symbol {
            symbol_type: Type::None,
            name_checksum: NameChecksum::None,
            value: Value::None,
        }
    }

    pub fn from_reader<R: BufRead>(
        reader: &mut R,
    ) -> Result<Self, StructureError> {
        let type_byte = reader.read_u8()?;

        // 8-bit / 16-bit mask in bits 6/7
        let type_byte_masked =
            type_byte & (!CHECKSUM_LOOKUP_MASK_8) & (!CHECKSUM_LOOKUP_MASK_16);

        let use_lookup_8 = (type_byte & CHECKSUM_LOOKUP_MASK_8) > 0;
        let use_lookup_16 = (type_byte & CHECKSUM_LOOKUP_MASK_16) > 0;

        // Error if both checksum bits are set
        (!(use_lookup_8 && use_lookup_16))
            .then(|| ())
            .ok_or(StructureError::BothChecksumBits(type_byte))?;

        let symbol_type = Type::try_from(type_byte_masked)?;

        let name_checksum = NameChecksum::from_reader(
            reader,
            symbol_type,
            use_lookup_8,
            use_lookup_16,
        )?;

        let value = Value::from_reader(reader, symbol_type)?;

        let symbol = Symbol {
            symbol_type,
            name_checksum,
            value,
        };

        Ok(symbol)
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StructureError> {
        let type_byte = self.symbol_type as u8;

        let type_byte_with_checksum_bits = match self.name_checksum {
            NameChecksum::Compressed16(_) => {
                type_byte | CHECKSUM_LOOKUP_MASK_16
            }
            NameChecksum::Compressed8(_) => type_byte | CHECKSUM_LOOKUP_MASK_8,
            _ => type_byte,
        };

        writer.write_u8(type_byte_with_checksum_bits)?;
        self.name_checksum.write(writer)?;
        self.value.write(writer)?;

        Ok(())
    }

    pub fn name(&self) -> Option<String> {
        self.name_checksum.to_name()
    }

    pub fn has_name_checksum(&self, name_checksum: NameChecksum) -> bool {
        self.name_checksum == name_checksum
    }

    pub fn has_name(&self, name: &String) -> bool {
        self.name_checksum
            .to_name()
            .filter(|lookup_name| lookup_name == name)
            .is_some()
    }

    pub fn with_value(&self, value: Value) -> Self {
        Self {
            symbol_type: self.symbol_type,
            name_checksum: self.name_checksum,
            value,
        }
    }

    pub fn symbol_type(&self) -> Type {
        self.symbol_type
    }

    pub fn name_checksum(&self) -> NameChecksum {
        self.name_checksum
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn try_as_struct(&self) -> Result<Arc<Structure>, StructureError> {
        match &self.value {
            Value::Structure(structure) => Ok(Arc::clone(structure)),
            _ => Err(StructureError::NotAStructure(self.name_checksum)),
        }
    }
}
