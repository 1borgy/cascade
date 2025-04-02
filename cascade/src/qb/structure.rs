use std::{
    fmt::Debug,
    io::{Read, Write},
};

use serde::{ser::SerializeSeq, Serialize, Serializer};

use super::Id;
use crate::qb::{Error, Kind, Symbol};

#[derive(Debug, Clone)]
pub struct Structure {
    symbols: Vec<Symbol>,
}

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for symbol in &self.symbols {
            seq.serialize_element(&symbol)?;
        }

        seq.end()
    }
}

impl Structure {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        // TODO: check duplicate symbols?
        Self {
            symbols, //: symbols
                     //.into_iter()
                     //.map(|symbol| (symbol.id, symbol))
                     //.collect(),
        }
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
        for symbol in &self.symbols {
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
        self.symbols.iter().filter(|symbol| symbol.id == id).next()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Symbol> {
        self.symbols
            .iter_mut()
            .filter(|symbol| symbol.id == id)
            .next()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn insert(&mut self, symbol: Symbol) -> Option<Symbol> {
        match self.get_mut(symbol.id) {
            Some(existing) => {
                let ret = existing.clone();
                *existing = symbol;
                Some(ret)
            }
            None => {
                self.symbols.push(symbol);
                None
            }
        }
    }

    pub fn remove(&mut self, id: Id) {
        self.symbols.retain(|symbol| symbol.id != id);
    }
}

impl FromIterator<Symbol> for Structure {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
