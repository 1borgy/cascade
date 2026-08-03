use std::sync::LazyLock;

use crc::{Algorithm, Crc};

static CRC: LazyLock<Crc<u32>> = LazyLock::new(|| {
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
