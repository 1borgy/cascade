use std::io::{Read, Seek, Write};

pub trait Entry {
    type Error;

    fn reader(&self) -> Result<impl Read + Seek, Self::Error>;
    fn writer(&self) -> Result<impl Write + Seek, Self::Error>;
    fn name(&self) -> String;
    fn rewrite_metadata(&self) -> Result<(), Self::Error>;
}
