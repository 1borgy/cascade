use cascade_dump as dump;
use cascade_lut::Lut;
use serde::{Deserialize, Serialize};

use crate::{Ska, ska::Header};

#[derive(Serialize, Deserialize)]
pub struct Dump {
    header: Header,
    summary: dump::Structure,
    data: dump::Structure,
}

impl Dump {
    pub fn new(ska: &Ska, lut: &Lut) -> Self {
        Self {
            header: ska.header.clone(),
            summary: dump::Structure::new(&*ska.summary, lut),
            data: dump::Structure::new(&*ska.data, lut),
        }
    }
}
