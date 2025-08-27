use std::io::{Read, Seek, Write};

use crate::{Error, Result};

pub struct Save(cascade_save::Save);

impl cascade_core::Save for Save {
    type Error = Error;

    fn read(reader: &mut (impl Read + Seek)) -> Result<Self> {
        Ok(Self(cascade_save::Save::read(reader)?))
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        Ok(self.0.write(writer)?)
    }
}

impl std::ops::Deref for Save {
    type Target = cascade_save::Save;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Save {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
