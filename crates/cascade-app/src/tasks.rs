use std::path::Path;

use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

use crate::Result;

pub async fn write_ron(obj: impl Serialize, to: impl AsRef<Path>) -> Result<usize> {
    let mut file = fs::File::create(&to).await?;

    let contents = ron::ser::to_string(&obj)?;

    let bytes = file.write(&contents.as_bytes()).await?;

    log::info!("wrote {} bytes to {:?}", bytes, to.as_ref());

    Ok(bytes)
}
