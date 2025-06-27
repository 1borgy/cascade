use std::{fmt, io::Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::Result;

pub struct Entry {
    pub offset: u32,
    pub size: u32,
    pub path: String,
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Entry {{ offset={:#08x}, size={:#04x}, path=\"{}\" }}",
            self.offset, self.size, self.path
        )
    }
}

#[derive(Debug)]
pub struct File {
    pub entries: Vec<Entry>,
}

impl File {
    pub fn read(reader: &mut impl Read) -> Result<Self> {
        let mut entries = Vec::new();

        while {
            match Entry::read(reader)? {
                Some(entry) => {
                    entries.push(entry);
                    true
                }
                None => false,
            }
        } {}

        Ok(Self { entries })
    }
}

impl Entry {
    pub fn read(reader: &mut impl Read) -> Result<Option<Self>> {
        let offset = reader.read_u32::<LittleEndian>()?;
        if offset == 0xFFFFFFFF {
            Ok(None)
        } else {
            let size = reader.read_u32::<LittleEndian>()?;

            let path = Self::read_path(reader)?;
            Ok(Some(Self { offset, size, path }))
        }
    }

    fn read_path(reader: &mut impl Read) -> Result<String> {
        let mut path = Vec::new();

        while {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;

            let path_bytes: Vec<u8> = buffer.into_iter().filter(|byte| *byte != 0).collect();
            let any_bytes_0 = path_bytes.len() < 4;

            path.extend(path_bytes);

            !any_bytes_0
        } {}

        Ok(str::from_utf8(&path[..])?.to_string())
    }
}
