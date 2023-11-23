use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

// I feel like this whole module is messy but it gets the job done ig

const CHECKSUM_LOOKUP_BYTES: &[u8] =
    include_bytes!("../../resources/checksum_lookup.yaml");
const COMPRESSED_LOOKUP_BYTES: &[u8] =
    include_bytes!("../../resources/compressed_lookup.yaml");

lazy_static! {
    static ref CHECKSUM_LOOKUP: HashMap<usize, String> = load_checksum_lookup();
    static ref COMPRESSED_LOOKUP: CompressedLookupTable =
        load_compressed_lookup();
    static ref REVERSE_CHECKSUM_LOOKUP: HashMap<String, u32> =
        create_reverse_checksum_lookup(&CHECKSUM_LOOKUP);
    static ref REVERSE_COMPRESSED_LOOKUP: ReverseCompressedLookupTable =
        create_reverse_compressed_lookup(&COMPRESSED_LOOKUP);
}

#[derive(Serialize, Deserialize)]
struct CompressedLookupTable {
    byte: Vec<String>,
    word: Vec<String>,
}

struct ReverseCompressedLookupTable {
    byte: HashMap<String, u8>,
    word: HashMap<String, u16>,
}

// .expect() is lazy here but we can guarantee it's valid YAML since we provide it
// (and is critical for execution)
fn load_checksum_lookup() -> HashMap<usize, String> {
    serde_yaml::from_slice(CHECKSUM_LOOKUP_BYTES)
        .expect("could not load checksum lookup table!")
}

fn create_reverse_checksum_lookup(
    checksum_lookup: &HashMap<usize, String>,
) -> HashMap<String, u32> {
    let mut reverse_lookup = HashMap::new();

    for (checksum, name) in checksum_lookup.iter() {
        reverse_lookup.insert(name.clone(), *checksum as u32);
    }

    reverse_lookup
}

fn load_compressed_lookup() -> CompressedLookupTable {
    serde_yaml::from_slice(COMPRESSED_LOOKUP_BYTES)
        .expect("could not load compressed lookup table!")
}

fn create_reverse_compressed_lookup(
    compressed_lookup: &CompressedLookupTable,
) -> ReverseCompressedLookupTable {
    let mut byte = HashMap::new();
    let mut word = HashMap::new();

    for (compressed_index, name) in compressed_lookup.byte.iter().enumerate() {
        byte.insert(name.clone(), compressed_index as u8);
    }

    for (compressed_index, name) in compressed_lookup.word.iter().enumerate() {
        word.insert(name.clone(), compressed_index as u16);
    }

    ReverseCompressedLookupTable { byte, word }
}

pub fn checksum(checksum: u32) -> Option<String> {
    CHECKSUM_LOOKUP.get(&(checksum as usize)).cloned()
}

pub fn compressed8(byte: u8) -> Option<String> {
    COMPRESSED_LOOKUP.byte.get(byte as usize).cloned()
}

pub fn compressed16(word: u16) -> Option<String> {
    COMPRESSED_LOOKUP.word.get(word as usize).cloned()
}

pub fn reverse_checksum(name: &String) -> Option<u32> {
    REVERSE_CHECKSUM_LOOKUP.get(name).cloned()
}

pub fn reverse_compressed16(name: &String) -> Option<u16> {
    REVERSE_COMPRESSED_LOOKUP.word.get(name).cloned()
}

pub fn reverse_compressed8(name: &String) -> Option<u8> {
    REVERSE_COMPRESSED_LOOKUP.byte.get(name).cloned()
}
