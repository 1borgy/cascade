use std::{
    backtrace::Backtrace,
    fmt::{Debug, Display},
    io,
    io::{Read, Write},
    rc::Rc,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

use crate::lookup::{
    lookup_checksum, lookup_compressed_byte, lookup_compressed_word,
};

const CHECKSUM_LOOKUP_MASK_8: u8 = 1 << 7;
const CHECKSUM_LOOKUP_MASK_16: u8 = 1 << 6;

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("invalid symbol type {0}")]
    InvalidType(u8),

    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error("internal error: {0} is not implemented")]
    NotImplemented(String),

    #[error(
        "both checksum table lookup bits set in symbol type byte: {0:#02x}"
    )]
    BothChecksumBits(u8),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    None = 0,
    Integer,
    Float,
    String,
    LocalString,
    Pair,
    Vector,
    QScript,
    CFunction,
    MemberFunction,
    Structure,
    StructurePointer,
    Array,
    Name,
    I8,
    I16,
    U8,
    U16,
    ZeroInt,
    ZeroFloat,
}

impl TryFrom<u8> for Type {
    type Error = SymbolError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Type::None as u8 => Ok(Type::None),
            x if x == Type::Integer as u8 => Ok(Type::Integer),
            x if x == Type::Float as u8 => Ok(Type::Float),
            x if x == Type::String as u8 => Ok(Type::String),
            x if x == Type::LocalString as u8 => Ok(Type::LocalString),
            x if x == Type::Pair as u8 => Ok(Type::Pair),
            x if x == Type::Vector as u8 => Ok(Type::Vector),
            x if x == Type::QScript as u8 => Ok(Type::QScript),
            x if x == Type::CFunction as u8 => Ok(Type::CFunction),
            x if x == Type::MemberFunction as u8 => Ok(Type::MemberFunction),
            x if x == Type::Structure as u8 => Ok(Type::Structure),
            x if x == Type::StructurePointer as u8 => {
                Ok(Type::StructurePointer)
            }
            x if x == Type::Array as u8 => Ok(Type::Array),
            x if x == Type::Name as u8 => Ok(Type::Name),
            x if x == Type::I8 as u8 => Ok(Type::I8),
            x if x == Type::I16 as u8 => Ok(Type::I16),
            x if x == Type::U8 as u8 => Ok(Type::U8),
            x if x == Type::U16 as u8 => Ok(Type::U16),
            x if x == Type::ZeroInt as u8 => Ok(Type::ZeroInt),
            x if x == Type::ZeroFloat as u8 => Ok(Type::ZeroFloat),
            _ => Err(SymbolError::InvalidType(v)),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Deserialize)]
pub enum NameChecksum {
    None,
    Inline(u32),
    Compressed8(u8),
    Compressed16(u16),
}

impl Serialize for NameChecksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            self.resolve().unwrap_or(format!("{:?}", self)).as_str(),
        )
    }
}

impl Debug for NameChecksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameChecksum::None => write!(f, "None"),
            NameChecksum::Inline(val) => write!(f, "Inline(0x{:x})", val),
            NameChecksum::Compressed8(val) => {
                write!(f, "Compressed8(0x{:x})", val)
            }
            NameChecksum::Compressed16(val) => {
                write!(f, "Compressed16(0x{:x})", val)
            }
        }
    }
}

impl NameChecksum {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        symbol_type: Type,
        use_lookup_8: bool,
        use_lookup_16: bool,
    ) -> Result<NameChecksum, SymbolError> {
        Ok(if symbol_type == Type::None {
            NameChecksum::None
        } else if use_lookup_8 {
            NameChecksum::Compressed8(reader.read_u8()?)
        } else if use_lookup_16 {
            NameChecksum::Compressed16(reader.read_u16::<LittleEndian>()?)
        } else {
            NameChecksum::Inline(reader.read_u32::<LittleEndian>()?)
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SymbolError> {
        match self {
            NameChecksum::Inline(val) => {
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

    pub fn resolve(&self) -> Option<String> {
        match self {
            NameChecksum::Inline(checksum) => lookup_checksum(*checksum),
            NameChecksum::Compressed8(byte) => lookup_compressed_byte(*byte),
            NameChecksum::Compressed16(word) => lookup_compressed_word(*word),
            NameChecksum::None => Some("".to_string()),
        }
    }
}

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
    String(String),
    Pair(f32, f32),
    Vector(f32, f32, f32),
    Structure(Rc<Structure>),
    Array(Type, Vec<Value>),
    Name(u32),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Value {
    fn from_reader<R: Read>(
        reader: &mut R,
        symbol_type: Type,
    ) -> Result<Value, SymbolError> {
        Ok(match symbol_type {
            Type::None => Value::None,
            Type::Integer => Value::I32(reader.read_i32::<LittleEndian>()?),
            Type::Float => Value::F32(reader.read_f32::<LittleEndian>()?),
            Type::String | Type::LocalString => {
                let mut value = String::new();

                while {
                    // do:
                    let character = reader.read_u8()? as char;
                    let is_null = character == '\0';

                    if !is_null {
                        value.push(character);
                    }

                    // while:
                    !is_null
                } {}
                Value::String(value)
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
                Value::Structure(Rc::new(Structure::from_reader(reader)?))
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

                Value::Array(symbol_type, elements)
            }
            Type::Name => Value::Name(reader.read_u32::<LittleEndian>()?),
            Type::I8 => Value::I8(reader.read_i8()?),
            Type::I16 => Value::I16(reader.read_i16::<LittleEndian>()?),
            Type::U8 => Value::U8(reader.read_u8()?),
            Type::U16 => Value::U16(reader.read_u16::<LittleEndian>()?),
            Type::ZeroInt => Value::ZeroInt,
            Type::ZeroFloat => Value::ZeroFloat,
            _ => {
                return Err(SymbolError::NotImplemented(format!(
                    "deserializing symbol type {:?}",
                    symbol_type
                )))
            }
        })
    }
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SymbolError> {
        match self {
            Value::U8(val) => writer.write_u8(*val)?,
            Value::U16(val) => writer.write_u16::<LittleEndian>(*val)?,
            Value::I8(val) => writer.write_i8(*val)?,
            Value::I16(val) => writer.write_i16::<LittleEndian>(*val)?,
            Value::I32(val) => writer.write_i32::<LittleEndian>(*val)?,
            Value::F32(val) => writer.write_f32::<LittleEndian>(*val)?,
            Value::String(val) => {
                writer.write_all(val.as_bytes())?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol_type: Type,
    pub name_checksum: NameChecksum,
    pub value: Value,
}

impl Symbol {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, SymbolError> {
        let type_byte = reader.read_u8()?;

        // 8-bit / 16-bit mask in bits 6/7
        let type_byte_masked =
            type_byte & (!CHECKSUM_LOOKUP_MASK_8) & (!CHECKSUM_LOOKUP_MASK_16);

        let use_lookup_8 = (type_byte & CHECKSUM_LOOKUP_MASK_8) > 0;
        let use_lookup_16 = (type_byte & CHECKSUM_LOOKUP_MASK_16) > 0;

        if use_lookup_8 & use_lookup_16 {
            return Err(SymbolError::BothChecksumBits(type_byte));
        }

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

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SymbolError> {
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

    pub fn has_name(&self, name: &str) -> bool {
        self.name_checksum
            .resolve()
            .filter(|lookup_name| lookup_name == name)
            .is_some()
    }

    pub fn name(&self) -> Option<String> {
        self.name_checksum.resolve()
    }

    pub fn with_value(&self, value: Value) -> Symbol {
        Symbol {
            symbol_type: self.symbol_type,
            name_checksum: self.name_checksum.clone(),
            value,
        }
    }

    pub fn try_as_struct(&self) -> Result<Rc<Structure>, SymbolError> {
        match &self.value {
            Value::Structure(structure) => Ok(Rc::clone(&structure)),
            // TODO make error type for this
            _ => Err(SymbolError::NotImplemented(format!(
                "could not get symbol as struct, symbol was {:?}",
                self
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    symbols: Vec<Rc<Symbol>>,
}

impl Structure {
    pub fn try_get(&self, name: &str) -> Result<Rc<Symbol>, SymbolError> {
        Ok(Rc::clone(
            self.symbols
                .iter()
                .filter(|symbol| {
                    log::debug!(
                        "checking symbol {:?} for name {}",
                        symbol.name(),
                        name
                    );
                    symbol.has_name(name)
                })
                .next()
                // TODO make error type for this
                .ok_or(SymbolError::NotImplemented(format!(
                    "could not get {} symbol in struct",
                    name
                )))?,
        ))
    }

    /// maybe won't actually replace if the symbol name doesnt match
    pub fn with_replaced_symbol(
        &self,
        name: &str,
        new: Rc<Symbol>,
    ) -> Result<Rc<Structure>, SymbolError> {
        log::debug!("replacing {} with {:?}", name, new);

        let mut symbol_found = false;

        let new_struct = Rc::new(Structure {
            symbols: self
                .symbols
                .iter()
                .map(|symbol| {
                    if symbol.has_name(name) {
                        symbol_found = true;
                        Rc::clone(&new)
                    } else {
                        Rc::clone(&symbol)
                    }
                })
                .collect(),
        });

        symbol_found
            .then(|| new_struct)
            // TODO make error type for this
            .ok_or(SymbolError::NotImplemented(format!(
                "could not replace symbol {}",
                name
            )))
    }

    pub fn try_get_nested_symbol(
        &self,
        names: &[&str],
    ) -> Result<Rc<Symbol>, SymbolError> {
        log::debug!("trying to get nested {:?}", names);

        match names {
            // TODO make error type for this
            // no names left: internal error, should never happen
            [] => Err(SymbolError::NotImplemented(
                "internal error: no more names for getting nested symbol"
                    .to_string(),
            )),

            // last name in the nested path: expected base case
            [name] => Ok(self.try_get(name)?),

            // more names to go, so recurse
            [name, ..] => {
                let symbol = self.try_get(name)?;
                Ok(symbol
                    .try_as_struct()?
                    .try_get_nested_symbol(&names[1..])?)
            }
        }
    }

    pub fn with_replaced_nested_symbol(
        &self,
        names: &[&str],
        new: Rc<Symbol>,
    ) -> Result<Rc<Structure>, SymbolError> {
        log::debug!("trying to replace nested {:?}", names);

        match names {
            // TODO make error type for this
            // no names left: internal error, should never happen
            [] => Err(SymbolError::NotImplemented("no names left".to_string())),

            // last name in the nested path: expected base case
            [name] => Ok(self.with_replaced_symbol(name, new)?),

            // more names to go, so recurse
            [name, ..] => {
                let inner_symbol = self.try_get(name)?;
                let inner_struct = inner_symbol.try_as_struct()?;

                Ok(self.with_replaced_symbol(
                    name,
                    Rc::new(inner_symbol.with_value(
                        Value::Structure(
                            inner_struct.with_replaced_nested_symbol(
                                &names[1..],
                                new,
                            )?,
                        ),
                    )),
                )?)
            }
        }
    }

    pub fn with_copied_path(
        &self,
        other: &Structure,
        names: &[&str],
    ) -> Result<Rc<Structure>, SymbolError> {
        match self.with_replaced_nested_symbol(
            &names,
            other.try_get_nested_symbol(&names)?,
        ) {
            Ok(structure) => Ok(structure),
            Err(err) => {
                log::error!("could not find path {:?}: {}", names, err);
                Err(err)
            }
        }
    }

    pub fn from_reader<R: Read>(
        reader: &mut R,
    ) -> Result<Structure, SymbolError> {
        let mut symbols = vec![];

        while {
            // do:
            let symbol = Symbol::from_reader(reader)?;
            let symbol_type = symbol.symbol_type;

            symbols.push(Rc::new(symbol));

            // while:
            match symbol_type {
                Type::None => false,
                _ => true,
            }
        } {}

        Ok(Structure { symbols })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SymbolError> {
        for symbol in self.symbols.iter() {
            symbol.write(writer)?;
        }

        Ok(())
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>, SymbolError> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;

        Ok(bytes)
    }
}
