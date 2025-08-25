use cascade_dump as dump;
use cascade_lut::Lut;

use crate::{chunk, save};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rethawed {
    magic: u32,
    version: u32,
    chunk_count: u32,
    chunks: Vec<Chunk>,
}

impl Rethawed {
    pub fn new(save: &save::Rethawed, lut: &Lut) -> Self {
        Self {
            magic: save.magic,
            version: save.version,
            chunk_count: save.chunk_count,
            chunks: save
                .chunks
                .iter()
                .map(|chunk| Chunk::new(chunk, lut))
                .collect(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Save {
    Thaw(dump::Save),
    Rethawed(Rethawed),
}

impl Save {
    pub fn new(save: &save::Save, lut: &Lut) -> Self {
        match save {
            save::Save::Thaw(save) => Self::Thaw(dump::Save::new(save, lut)),
            save::Save::Rethawed(save) => Self::Rethawed(Rethawed::new(save, lut)),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chunk {
    magic: chunk::Magic,
    size: u32,
    structure: dump::Structure,
}

impl Chunk {
    pub fn new(chunk: &chunk::Chunk, lut: &Lut) -> Self {
        Self {
            magic: chunk.magic,
            size: chunk.size,
            structure: dump::Structure::new(&chunk.structure, lut),
        }
    }
}
