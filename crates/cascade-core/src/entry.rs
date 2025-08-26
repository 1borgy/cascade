use std::io::{Read, Seek, Write};

pub trait Entry<Error> {
    fn reader<R>(&self) -> Result<R, Error>
    where
        R: Read + Seek;

    fn writer<W>(&self) -> Result<W, Error>
    where
        W: Write + Seek;

    fn name(&self) -> String;

    fn rewrite_metadata(&self) -> Result<(), Error>;
}
