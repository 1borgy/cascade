use crc::{Algorithm, Crc};
use lazy_static::lazy_static;

lazy_static! {
    static ref CRC_ALG: Algorithm<u32> = Algorithm {
        width: 32,
        poly: 0x04c11db7,
        init: 0xffffffff,
        refin: true,
        refout: true,
        xorout: 0x0000,
        check: 0xaee7,
        residue: 0x0000,
    };
    static ref CRC: Crc<u32> = Crc::<u32>::new(&CRC_ALG);
}

pub fn get_checksum_for_bytes(bytes: &Vec<u8>) -> u32 {
    CRC.checksum(bytes.as_slice())
}
