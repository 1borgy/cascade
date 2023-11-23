use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, Write},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use super::NameChecksum;
use crate::structure::{StructureError, Symbol, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    symbols: Vec<Symbol>,

    index_lookup: HashMap<NameChecksum, usize>,
}

impl Structure {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        // TODO: Should duplicate symbols be checked?
        let index_lookup = symbols
            .iter()
            .enumerate()
            .map(|(index, symbol)| (symbol.name_checksum().clone(), index))
            .collect();

        Self {
            symbols,
            index_lookup,
        }
    }

    pub fn from_reader<R: BufRead>(
        reader: &mut R,
    ) -> Result<Structure, StructureError> {
        let mut symbols = vec![];

        while {
            // do:
            // Read symbol from the reader
            let symbol = Symbol::from_reader(reader)?;
            let symbol_type = symbol.symbol_type();

            // while:
            // The symbol is not none
            match symbol_type {
                Type::None => false,
                _ => {
                    // Only push the symbol if it is non-none
                    symbols.push(symbol);
                    true
                }
            }
        } {}

        Ok(Self::new(symbols))
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StructureError> {
        for symbol in self.symbols.iter() {
            symbol.write(writer)?;
        }

        // Each structure is terminated with a none symbol
        Symbol::none().write(writer)?;

        Ok(())
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>, StructureError> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;

        Ok(bytes)
    }

    pub fn index(&self, name_checksum: NameChecksum) -> Option<usize> {
        self.index_lookup.get(&name_checksum).cloned()
    }

    pub fn try_index<N>(&self, name: N) -> Result<Option<usize>, StructureError>
    where
        N: TryInto<NameChecksum, Error = StructureError>,
    {
        Ok(self.index(name.try_into()?))
    }

    pub fn get(&self, name_checksum: NameChecksum) -> Option<&Symbol> {
        self.index(name_checksum)
            .and_then(|index| self.symbols.get(index))
    }

    pub fn try_get<N>(&self, name: N) -> Result<Option<&Symbol>, StructureError>
    where
        N: TryInto<NameChecksum, Error = StructureError>,
    {
        Ok(self.get(name.try_into()?))
    }

    pub fn has(&self, name_checksum: NameChecksum) -> bool {
        self.index(name_checksum).is_some()
    }

    pub fn try_has<N>(&self, name: N) -> Result<bool, StructureError>
    where
        N: TryInto<NameChecksum, Error = StructureError>,
    {
        Ok(self.has(name.try_into()?))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }
}

impl FromIterator<Symbol> for Structure {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl FromIterator<Symbol> for Arc<Structure> {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
