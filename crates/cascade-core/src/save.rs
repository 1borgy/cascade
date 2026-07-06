use std::io::{Read, Seek, Write};

use crate::Result;

pub trait Save: Sized {
    fn read(reader: &mut (impl Read + Seek)) -> Result<Self>;
    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()>;
}
