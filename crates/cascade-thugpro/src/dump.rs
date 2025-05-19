use cascade_dump as dump;
use cascade_lut::Lut;
use serde::{Deserialize, Serialize};

use crate::save;

#[derive(Serialize, Deserialize)]
pub struct Dump {
    header: save::Header,
    summary: dump::Structure,
    data: dump::Structure,
}

impl Dump {
    pub fn new(file: &save::Ska, lut: &Lut) -> Self {
        Self {
            header: file.header.clone(),
            summary: dump::Structure::new(&*file.summary, lut),
            data: dump::Structure::new(&*file.data, lut),
        }
    }
}
