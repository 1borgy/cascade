use std::io::{Read, Seek, Write};

pub trait Save: Sized {
    type Error;

    fn read(reader: &mut (impl Read + Seek)) -> Result<Self, Self::Error>;
    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<(), Self::Error>;
}
