use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::structure::StructureError;

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
    type Error = StructureError;

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
            _ => Err(StructureError::InvalidType(v)),
        }
    }
}
