use cascade_lut::Lut;
use cascade_qb as qb;
use encoding_rs::WINDOWS_1252;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Kind();

#[derive(Serialize, Deserialize)]
pub enum Id {
    None,
    Checksum(u32),
    Compress8(u8),
    Compress16(u16),
}

impl From<&qb::Id> for Id {
    fn from(value: &qb::Id) -> Self {
        match value {
            qb::Id::None => Id::None,
            qb::Id::Checksum(v) => Id::Checksum(*v),
            qb::Id::Compress8(v) => Id::Compress8(*v),
            qb::Id::Compress16(v) => Id::Compress16(*v),
        }
    }
}

impl From<qb::Id> for Id {
    fn from(value: qb::Id) -> Self {
        Id::from(&value)
    }
}

#[derive(Serialize, Deserialize)]
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
    Structure(Box<Structure>),
    Array(Vec<Value>),
    Name(Option<String>),
}

impl Value {
    fn new(value: &qb::Value, lut: &Lut) -> Self {
        match value {
            qb::Value::None => Value::None,
            qb::Value::U8(v) => Value::U8(*v),
            qb::Value::U16(v) => Value::U16(*v),
            qb::Value::I8(v) => Value::I8(*v),
            qb::Value::I16(v) => Value::I16(*v),
            qb::Value::I32(v) => Value::I32(*v),
            qb::Value::F32(v) => Value::F32(*v),
            qb::Value::ZeroInt => Value::ZeroInt,
            qb::Value::ZeroFloat => Value::ZeroFloat,
            qb::Value::String(v) => {
                let (name, _, _) = WINDOWS_1252.decode(&v);
                Value::String(name.to_string())
            }
            qb::Value::Pair(x, y) => Value::Pair(*x, *y),
            qb::Value::Vector(x, y, z) => Value::Vector(*x, *y, *z),
            qb::Value::Structure(v) => Value::Structure(Box::new(Structure::new(&v, &lut))),
            qb::Value::Array(_, v) => {
                Value::Array(v.iter().map(|symbol| Value::new(&symbol, lut)).collect())
            }
            qb::Value::Name(v) => Value::Name(lut.checksum.lookup(*v).cloned()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Symbol {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub id: Id,
    pub value: Value,
}

impl Symbol {
    pub fn new(symbol: &qb::Symbol, lut: &Lut) -> Self {
        Self {
            name: match symbol.id {
                qb::Id::None => None,
                qb::Id::Checksum(v) => lut.checksum.lookup(v).cloned(),
                qb::Id::Compress8(v) => lut.compress.lookup8(v).cloned(),
                qb::Id::Compress16(v) => lut.compress.lookup16(v).cloned(),
            },
            id: symbol.id.into(),
            value: Value::new(&symbol.value, lut),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Structure(Vec<Symbol>);

impl Structure {
    pub fn new(structure: &qb::Structure, lut: &Lut) -> Self {
        let symbols = structure
            .iter()
            .map(|symbol| Symbol::new(symbol, lut))
            .collect();

        Self(symbols)
    }
}
