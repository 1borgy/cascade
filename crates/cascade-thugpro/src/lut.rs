use cascade_lut as lut;

const COMPRESS_LUT_BYTES: &[u8] = include_bytes!("../../../assets/lut/compress-thugpro.ron");

pub fn load_compress() -> lut::Result<lut::Compress> {
    lut::Compress::from_bytes(COMPRESS_LUT_BYTES)
}
