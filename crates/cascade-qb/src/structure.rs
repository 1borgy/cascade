use std::{
    fmt::Debug,
    io::{Read, Write},
};

use crate::{Error, Id, Kind, Symbol};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Structure(Vec<Symbol>);

impl Structure {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self(symbols)
    }

    pub fn read(reader: &mut impl Read) -> Result<Structure, Error> {
        let mut symbols = vec![];

        while {
            // do:
            // Read symbol from the reader
            let symbol = Symbol::read(reader)?;
            let kind = symbol.kind;

            // while:
            // The symbol is not none
            match kind {
                Kind::None => false,
                _ => {
                    // Only push the symbol if it is non-none
                    symbols.push(symbol);
                    true
                }
            }
        } {}

        Ok(Self::new(symbols))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for symbol in &self.0 {
            symbol.write(writer)?;
        }

        // Each structure is terminated with a none symbol
        Symbol::none().write(writer)?;

        Ok(())
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;

        Ok(bytes)
    }

    pub fn get(&self, id: Id) -> Option<&Symbol> {
        self.0.iter().filter(|symbol| symbol.id == id).next()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Symbol> {
        self.0.iter_mut().filter(|symbol| symbol.id == id).next()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, symbol: Symbol) -> Option<Symbol> {
        match self.get_mut(symbol.id) {
            Some(existing) => {
                let ret = existing.clone();
                *existing = symbol;
                Some(ret)
            }
            None => {
                self.0.push(symbol);
                None
            }
        }
    }

    pub fn remove(&mut self, id: Id) {
        self.0.retain(|symbol| symbol.id != id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.0.iter()
    }
}

impl FromIterator<Symbol> for Structure {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
