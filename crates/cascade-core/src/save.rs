use std::io::{Read, Seek, Write};

pub trait Save<Error> {
    fn read<R>(&self, reader: &mut R) -> Result<impl Save<Error>, Error>
    where
        R: Read + Seek;

    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write + Seek;
}
