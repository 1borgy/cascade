use std::{io, path::Path};

use ron::de::SpannedError;
use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

use crate::config::Format;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(io::ErrorKind),

    #[error("ron serialization error: {0}")]
    Ron(#[from] ron::Error),

    #[error("toml serialization error: {0}")]
    Toml(#[from] toml::ser::Error),

    #[error("ron deserialization error: {0}")]
    Spanned(#[from] SpannedError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

pub async fn write(
    obj: impl Serialize,
    to: impl AsRef<Path>,
    format: Format,
) -> Result<usize, Error> {
    let mut file = fs::File::create(&to).await?;

    let contents = match format {
        Format::Ron => ron::ser::to_string(&obj)?,
        Format::Toml => toml::ser::to_string_pretty(&obj)?,
    };

    let bytes = file.write(&contents.as_bytes()).await?;

    log::info!("wrote {} bytes to {:?}", bytes, to.as_ref());

    Ok(bytes)
}
