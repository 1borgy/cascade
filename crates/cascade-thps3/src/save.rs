use std::io::{BufRead, BufReader, Read, Seek, Write};

use cascade_core::Result;
use encoding_rs::WINDOWS_1252;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Save {
    pub filename: String,
    pub structure: cascade_qb::Structure,
    pub symbols: Vec<cascade_qb::Symbol>,
}

impl cascade_core::Save for Save {
    fn read(reader: &mut (impl Read + Seek)) -> Result<Self> {
        let mut reader = BufReader::new(reader);

        let mut filename_bytes = Vec::new();
        reader.read_until(0, &mut filename_bytes)?;

        let (filename, _, _) = WINDOWS_1252.decode(&filename_bytes[..filename_bytes.len() - 1]); // Strip null terminator
        let filename = filename.to_string();
        let structure = cascade_qb::Structure::read(&mut reader)?;

        let mut symbols = Vec::new();
        while let Ok(symbol) = cascade_qb::Symbol::read(&mut reader) {
            symbols.push(symbol);
        }

        Ok(Self {
            filename,
            structure,
            symbols,
        })
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        let (filename_bytes, _, _) = WINDOWS_1252.encode(self.filename.as_str());

        writer.write_all(&filename_bytes)?;
        writer.write_all(&[0])?; // Write null terminator
        self.structure.write(writer)?;

        Ok(())
    }
}
