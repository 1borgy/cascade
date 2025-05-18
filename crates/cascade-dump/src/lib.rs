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
            cascade_qb::Id::None => Id::None,
            cascade_qb::Id::Checksum(v) => Id::Checksum(*v),
            cascade_qb::Id::Compress8(v) => Id::Compress8(*v),
            cascade_qb::Id::Compress16(v) => Id::Compress16(*v),
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
            cascade_qb::Value::None => Value::None,
            cascade_qb::Value::U8(v) => Value::U8(*v),
            cascade_qb::Value::U16(v) => Value::U16(*v),
            cascade_qb::Value::I8(v) => Value::I8(*v),
            cascade_qb::Value::I16(v) => Value::I16(*v),
            cascade_qb::Value::I32(v) => Value::I32(*v),
            cascade_qb::Value::F32(v) => Value::F32(*v),
            cascade_qb::Value::ZeroInt => Value::ZeroInt,
            cascade_qb::Value::ZeroFloat => Value::ZeroFloat,
            cascade_qb::Value::String(v) => {
                let (name, _, _) = WINDOWS_1252.decode(&v);
                Value::String(name.to_string())
            }
            cascade_qb::Value::Pair(x, y) => Value::Pair(*x, *y),
            cascade_qb::Value::Vector(x, y, z) => Value::Vector(*x, *y, *z),
            cascade_qb::Value::Structure(v) => Value::Structure(Box::new(Structure::new(&v, &lut))),
            cascade_qb::Value::Array(_, v) => {
                Value::Array(v.iter().map(|symbol| Value::new(&symbol, lut)).collect())
            }
            cascade_qb::Value::Name(v) => Value::Name(lut.checksum.lookup(*v).cloned()),
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
                cascade_qb::Id::None => None,
                cascade_qb::Id::Checksum(v) => lut.checksum.lookup(v).cloned(),
                cascade_qb::Id::Compress8(v) => lut.compress.lookup8(v).cloned(),
                cascade_qb::Id::Compress16(v) => lut.compress.lookup16(v).cloned(),
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
