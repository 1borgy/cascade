use std::io::{Read, Seek, Write};

// pub trait Parser {
//     type Save: Sized;
//     type Transform: Sized;
//     type Error: Send + Sync + std::error::Error;
//
//     fn read(&self, reader: &mut impl Read) -> Result<Self::Save, Self::Error>;
//     fn write(&self, save: &Self::Save, writer: &mut impl Write) -> Result<(), Self::Error>;
//     fn parse(&self, save: &Self::Save) -> Result<Self::Transform, Self::Error>;
//     fn modify(&self, save: &mut Self::Save, transform: Self::Transform) -> Result<(), Self::Error>;
// }

pub trait Explorer<Entry, Error> {
    fn list(&self) -> Result<impl Iterator<Item = Entry>, Error>;
    fn reader(&self, entry: &Entry) -> Result<impl Read + Seek, Error>;
    fn writer(&self, entry: &Entry) -> Result<impl Write, Error>;
    fn name(&self, entry: &Entry) -> String;
    fn rewrite_metadata(&self, entry: &Entry) -> Result<(), Error>;
}

pub struct Flags {
    pub summary: bool,
    pub trickset: bool,
    pub scales: bool,
}

pub trait Parser<Save, Cas, Error> {
    fn read(&self, reader: &mut (impl Read + Seek)) -> Result<Save, Error>;
    fn write(&self, save: &Save, writer: &mut impl Write) -> Result<(), Error>;
    fn parse(&self, save: &Save) -> Result<Cas, Error>;
    fn mask(&self, cas: Cas, flags: Flags) -> Cas;
    fn modify(&self, save: &mut Save, cas: &Cas) -> Result<(), Error>;
}
