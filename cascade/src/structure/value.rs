use std::{
    fmt::{self, Debug},
    io::{BufRead, Write},
    sync::Arc,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::structure::{Structure, StructureError, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    None,
    U8(u8),
    U16(u16),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    ZeroInt,
    ZeroFloat,
    String(Vec<u8>),
    Pair(f32, f32),
    Vector(f32, f32, f32),
    Structure(Arc<Structure>),
    Array(Type, Arc<Vec<Value>>),
    Name(u32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Value {
    pub fn from_reader<R: BufRead>(
        reader: &mut R,
        symbol_type: Type,
    ) -> Result<Value, StructureError> {
        Ok(match symbol_type {
            Type::None => Value::None,
            Type::Integer => Value::I32(reader.read_i32::<LittleEndian>()?),
            Type::Float => Value::F32(reader.read_f32::<LittleEndian>()?),
            Type::String | Type::LocalString => {
                let mut bytes = vec![];

                while {
                    // do:
                    // Read character
                    let byte = reader.read_u8()?;

                    // while:
                    if byte != 0 {
                        bytes.push(byte);
                        true
                    } else {
                        false
                    }
                } {}

                Value::String(bytes)
            }
            // https://doc.rust-lang.org/reference/expressions.html#evaluation-order-of-operands
            Type::Pair => Value::Pair(
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
            ),
            Type::Vector => Value::Vector(
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
            ),
            Type::Structure => {
                Value::Structure(Arc::new(Structure::from_reader(reader)?))
            }
            Type::Array => {
                // TODO this is code reuse w/ Symbol::deserialize
                let type_byte = reader.read_u8()?;
                let symbol_type = Type::try_from(type_byte)?;
                let len = reader.read_u16::<LittleEndian>()?;

                let mut elements = vec![];
                for _ in 0..len {
                    elements.push(Value::from_reader(reader, symbol_type)?)
                }

                Value::Array(symbol_type, Arc::new(elements))
            }
            Type::Name => Value::Name(reader.read_u32::<LittleEndian>()?),
            Type::I8 => Value::I8(reader.read_i8()?),
            Type::I16 => Value::I16(reader.read_i16::<LittleEndian>()?),
            Type::U8 => Value::U8(reader.read_u8()?),
            Type::U16 => Value::U16(reader.read_u16::<LittleEndian>()?),
            Type::ZeroInt => Value::ZeroInt,
            Type::ZeroFloat => Value::ZeroFloat,
            _ => {
                return Err(StructureError::NotImplemented(format!(
                    "deserializing symbol type {:?}",
                    symbol_type
                )))
            }
        })
    }
    pub fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StructureError> {
        match self {
            Value::U8(val) => writer.write_u8(*val)?,
            Value::U16(val) => writer.write_u16::<LittleEndian>(*val)?,
            Value::I8(val) => writer.write_i8(*val)?,
            Value::I16(val) => writer.write_i16::<LittleEndian>(*val)?,
            Value::I32(val) => writer.write_i32::<LittleEndian>(*val)?,
            Value::F32(val) => writer.write_f32::<LittleEndian>(*val)?,
            Value::String(val) => {
                writer.write_all(val)?;
                // null terminator
                writer.write_u8(0)?;
            }
            Value::Pair(a, b) => {
                writer.write_f32::<LittleEndian>(*a)?;
                writer.write_f32::<LittleEndian>(*b)?;
            }
            Value::Vector(a, b, c) => {
                writer.write_f32::<LittleEndian>(*a)?;
                writer.write_f32::<LittleEndian>(*b)?;
                writer.write_f32::<LittleEndian>(*c)?;
            }
            Value::Structure(structure) => structure.write(writer)?,
            Value::Array(symbol_type, values) => {
                writer.write_u8(*symbol_type as u8)?;
                writer.write_u16::<LittleEndian>(values.len() as u16)?;
                for value in values.iter() {
                    value.write(writer)?;
                }
            }
            Value::Name(val) => writer.write_u32::<LittleEndian>(*val)?,
            // no value is written for these types
            Value::None | Value::ZeroFloat | Value::ZeroInt => (),
        }
        Ok(())
    }
}
