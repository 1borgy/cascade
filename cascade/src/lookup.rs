use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const CHECKSUM_LOOKUP_BYTES: &[u8] =
    include_bytes!("../../resources/checksum_lookup.yaml");
const COMPRESSED_LOOKUP_BYTES: &[u8] =
    include_bytes!("../../resources/compressed_lookup.yaml");

lazy_static! {
    static ref CHECKSUM_LOOKUP: HashMap<usize, String> = load_checksum_lookup();
    static ref COMPRESSED_LOOKUP: CompressedLookupTable =
        load_compressed_lookup();
}

#[derive(Serialize, Deserialize)]
struct CompressedLookupTable {
    byte: Vec<String>,
    word: Vec<String>,
}

// .expect() is lazy here but we can guarantee it's valid YAML since we provide it
// (and is critical for execution)
fn load_checksum_lookup() -> HashMap<usize, String> {
    serde_yaml::from_slice(CHECKSUM_LOOKUP_BYTES)
        .expect("could not load checksum lookup table!")
}

fn load_compressed_lookup() -> CompressedLookupTable {
    serde_yaml::from_slice(COMPRESSED_LOOKUP_BYTES)
        .expect("could not load compressed lookup table!")
}

pub fn lookup_compressed_byte(byte: u8) -> Option<String> {
    COMPRESSED_LOOKUP.byte.get(byte as usize).cloned()
}

pub fn lookup_compressed_word(word: u16) -> Option<String> {
    COMPRESSED_LOOKUP.word.get(word as usize).cloned()
}

pub fn lookup_checksum(checksum: u32) -> Option<String> {
    CHECKSUM_LOOKUP.get(&(checksum as usize)).cloned()
}
