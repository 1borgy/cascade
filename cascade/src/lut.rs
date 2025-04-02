use std::{collections::HashMap, result, str::Utf8Error};

use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

use crate::qb;

const LUT_THUGPRO_BYTES: &[u8] =
    include_bytes!("../../resources/lut/thug_pro.ron");

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("ron deserialization error: {0}")]
    Spanned(#[from] SpannedError),

    #[error("utf8 decoding error: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("name not found in LUT: {0}")]
    NameNotFound(String),

    #[error("id not found in LUT: {0:?}")]
    IdNotFound(qb::Id),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Serialize, Deserialize)]
struct LutFile {
    checksum: HashMap<String, u32>,
    compressed_8: Vec<String>,
    compressed_16: Vec<String>,
}

impl LutFile {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let as_str = std::str::from_utf8(bytes)?;
        Ok(Self::from_str(as_str)?)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(ron::from_str(s)?)
    }

    pub fn thug_pro() -> Result<LutFile> {
        LutFile::from_bytes(LUT_THUGPRO_BYTES)
    }
}

#[derive(Debug, Clone)]
struct NameLut {
    checksum: HashMap<String, u32>,
    compressed_8: HashMap<String, u8>,
    compressed_16: HashMap<String, u16>,
}

impl From<&LutFile> for NameLut {
    fn from(file: &LutFile) -> Self {
        Self {
            checksum: file.checksum.clone(),
            compressed_8: file
                .compressed_8
                .iter()
                .enumerate()
                .map(|(index, name)| (name.clone(), index as u8))
                .collect(),
            compressed_16: file
                .compressed_16
                .iter()
                .enumerate()
                .map(|(index, name)| (name.clone(), index as u16))
                .collect(),
        }
    }
}

impl NameLut {
    pub fn lookup(&self, name: &impl ToString) -> Option<qb::Id> {
        let name = name.to_string();

        if let Some(checksum) = self.checksum.get(&name) {
            Some(qb::Id::Checksum(*checksum))
        } else if let Some(compressed8) = self.compressed_8.get(&name) {
            Some(qb::Id::Compressed8(*compressed8))
        } else if let Some(compressed16) = self.compressed_16.get(&name) {
            Some(qb::Id::Compressed16(*compressed16))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct IdLut {
    checksum: HashMap<u32, String>,
    compressed_8: HashMap<u8, String>,
    compressed_16: HashMap<u16, String>,
}

impl From<&LutFile> for IdLut {
    fn from(file: &LutFile) -> Self {
        Self {
            checksum: file
                .checksum
                .iter()
                .map(|(name, checksum)| (*checksum, name.clone()))
                .collect(),
            compressed_8: file
                .compressed_8
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, name)| (index as u8, name))
                .collect(),
            compressed_16: file
                .compressed_16
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, name)| (index as u16, name))
                .collect(),
        }
    }
}

impl IdLut {
    pub fn lookup(&self, id: qb::Id) -> Option<&String> {
        match id {
            qb::Id::Checksum(checksum) => self.checksum.get(&checksum),
            qb::Id::Compressed8(index) => self.compressed_8.get(&index),
            qb::Id::Compressed16(index) => self.compressed_16.get(&index),
            qb::Id::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lut {
    name: NameLut,
    id: IdLut,
}

impl From<LutFile> for Lut {
    fn from(file: LutFile) -> Self {
        Self {
            name: NameLut::from(&file),
            id: IdLut::from(&file),
        }
    }
}

impl Lut {
    pub fn by_name(&self, name: &impl ToString) -> Result<qb::Id> {
        self.name
            .lookup(name)
            .ok_or(Error::NameNotFound(name.to_string()))
    }

    pub fn by_id(&self, id: qb::Id) -> Result<&String> {
        self.id.lookup(id).ok_or(Error::IdNotFound(id))
    }

    pub fn thug_pro() -> Result<Lut> {
        Ok(Lut::from(LutFile::thug_pro()?))
    }
}
