use std::{
    fmt::{self, Debug},
    io::{Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{Error, Kind, Structure};

#[derive(Debug, Clone)]
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
    Structure(Box<Structure>),
    Array(Kind, Vec<Value>),
    Name(u32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Value {
    pub fn read(reader: &mut impl Read, kind: Kind) -> Result<Value, Error> {
        Ok(match kind {
            Kind::None => Value::None,
            Kind::Integer => Value::I32(reader.read_i32::<LittleEndian>()?),
            Kind::Float => Value::F32(reader.read_f32::<LittleEndian>()?),
            Kind::String | Kind::LocalString => {
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
            Kind::Pair => Value::Pair(
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
            ),
            Kind::Vector => Value::Vector(
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
                reader.read_f32::<LittleEndian>()?,
            ),
            Kind::Structure => Value::Structure(Box::new(Structure::read(reader)?)),
            Kind::Array => {
                // TODO this is code reuse w/ Symbol::deserialize
                let type_byte = reader.read_u8()?;
                let kind = Kind::try_from(type_byte)?;
                let len = reader.read_u16::<LittleEndian>()?;

                let mut elements = vec![];
                for _ in 0..len {
                    elements.push(Value::read(reader, kind)?)
                }

                Value::Array(kind, elements)
            }
            Kind::Name => Value::Name(reader.read_u32::<LittleEndian>()?),
            Kind::I8 => Value::I8(reader.read_i8()?),
            Kind::I16 => Value::I16(reader.read_i16::<LittleEndian>()?),
            Kind::U8 => Value::U8(reader.read_u8()?),
            Kind::U16 => Value::U16(reader.read_u16::<LittleEndian>()?),
            Kind::ZeroInt => Value::ZeroInt,
            Kind::ZeroFloat => Value::ZeroFloat,
            _ => {
                return Err(Error::NotImplemented(format!(
                    "deserializing symbol type {:?}",
                    kind
                )))
            }
        })
    }
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
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
            Value::Array(kind, values) => {
                writer.write_u8(*kind as u8)?;
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

    pub fn try_as_structure(self) -> Result<Box<Structure>, Error> {
        match self {
            Value::Structure(value) => Ok(value),
            value => Err(Error::ExpectedValueType(
                "Structure".to_string(),
                value.clone(),
            )),
        }
    }

    pub fn try_as_structure_mut(&mut self) -> Result<&mut Box<Structure>, Error> {
        match self {
            Value::Structure(value) => Ok(value),
            value => Err(Error::ExpectedValueType(
                "Structure".to_string(),
                value.clone(),
            )),
        }
    }
}
