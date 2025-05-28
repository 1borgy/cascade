use std::{collections::HashMap, result, str::Utf8Error};

use cascade_qb as qb;
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

const CHECKSUM_LUT_BYTES: &[u8] = include_bytes!("../../../assets/lut/checksum.ron");

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Checksum(pub HashMap<u32, String>);

impl Checksum {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let as_str = std::str::from_utf8(bytes)?;
        Ok(Self::from_str(as_str)?)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(ron::from_str(s)?)
    }

    pub fn lookup(&self, value: u32) -> Option<&String> {
        self.0.get(&value)
    }

    pub fn load() -> Result<Self> {
        Self::from_bytes(CHECKSUM_LUT_BYTES)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Compress {
    compress8: Vec<String>,
    compress16: Vec<String>,
}

impl Compress {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let as_str = std::str::from_utf8(bytes)?;
        Ok(Self::from_str(as_str)?)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(ron::from_str(s)?)
    }

    pub fn lookup8(&self, value: u8) -> Option<&String> {
        self.compress8.get(value as usize)
    }

    pub fn lookup16(&self, value: u16) -> Option<&String> {
        self.compress16.get(value as usize)
    }
}

pub struct Lut {
    pub checksum: Checksum,
    pub compress: Compress,
}
