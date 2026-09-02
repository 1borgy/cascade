use cascade_dump as dump;
use cascade_lut::Lut;

use crate::save;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Save {
    filename: String,
    structure: dump::Structure,
    symbols: Vec<dump::Symbol>,
}

impl Save {
    pub fn new(save: &save::Save, lut: &Lut) -> Self {
        Self {
            filename: save.filename.clone(),
            structure: dump::Structure::new(&save.structure, lut),
            symbols: save
                .symbols
                .iter()
                .map(|symbol| dump::Symbol::new(symbol, lut))
                .collect(),
        }
    }
}
