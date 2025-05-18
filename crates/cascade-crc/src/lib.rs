use std::cell::LazyCell;

use crc::{Algorithm, Crc};

const CRC: LazyCell<Crc<u32>> = LazyCell::new(|| {
    Crc::<u32>::new(&Algorithm {
        width: 32,
        poly: 0x04c11db7,
        init: 0xffffffff,
        refin: true,
        refout: true,
        xorout: 0x0000,
        check: 0xaee7,
        residue: 0x0000,
    })
});

pub fn checksum(bytes: &Vec<u8>) -> u32 {
    CRC.checksum(bytes.as_slice())
}
