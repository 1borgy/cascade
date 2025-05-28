use std::{collections::HashMap, fs, io, ops::Deref, path::Path};

use serde::{Deserialize, Serialize};

use crate::config::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selections(HashMap<String, bool>);

impl Selections {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;

        log::info!("reading selections from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = ron::from_str(contents.as_str())?;

        Ok(config)
    }
}

impl From<HashMap<String, bool>> for Selections {
    fn from(value: HashMap<String, bool>) -> Self {
        Self(value)
    }
}

impl Deref for Selections {
    type Target = HashMap<String, bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
