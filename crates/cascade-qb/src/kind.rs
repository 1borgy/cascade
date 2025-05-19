use std::fmt::Debug;

use crate::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Kind {
    None,
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

impl TryFrom<u8> for Kind {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Kind::None),
            1 => Ok(Kind::Integer),
            2 => Ok(Kind::Float),
            3 => Ok(Kind::String),
            4 => Ok(Kind::LocalString),
            5 => Ok(Kind::Pair),
            6 => Ok(Kind::Vector),
            7 => Ok(Kind::QScript),
            8 => Ok(Kind::CFunction),
            9 => Ok(Kind::MemberFunction),
            10 => Ok(Kind::Structure),
            11 => Ok(Kind::StructurePointer),
            12 => Ok(Kind::Array),
            13 => Ok(Kind::Name),
            14 => Ok(Kind::I8),
            15 => Ok(Kind::I16),
            16 => Ok(Kind::U8),
            17 => Ok(Kind::U16),
            18 => Ok(Kind::ZeroInt),
            19 => Ok(Kind::ZeroFloat),
            _ => Err(Error::InvalidType(v)),
        }
    }
}
