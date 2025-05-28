use cascade_dump as dump;
use cascade_lut::Lut;

use crate::save;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dump {
    #[allow(dead_code)]
    header: save::Header,
    #[allow(dead_code)]
    summary: dump::Structure,
    #[allow(dead_code)]
    data: dump::Structure,
}

impl Dump {
    pub fn new(file: &save::Save, lut: &Lut) -> Self {
        Self {
            header: file.header.clone(),
            summary: dump::Structure::new(&*file.summary, lut),
            data: dump::Structure::new(&*file.data, lut),
        }
    }
}
