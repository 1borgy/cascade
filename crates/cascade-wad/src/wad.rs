use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{Result, hed};

pub fn extract(hed: hed::File, wad: &mut fs::File, output_dir: impl AsRef<Path>) -> Result<()> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    for entry in hed.entries {
        match extract_entry(&entry, wad, output_dir) {
            Ok(_) => {
                log::info!("extracted entry {}", &entry.path);
            }
            Err(err) => {
                log::warn!("error extracting entry {}: {}", &entry.path, err);
            }
        }
    }

    Ok(())
}

fn extract_entry(
    entry: &hed::Entry,
    wad: &mut fs::File,
    output_dir: impl AsRef<Path>,
) -> Result<()> {
    let output_dir = output_dir.as_ref();
    let entry_path = output_dir.join(&entry.path.strip_prefix("\\").unwrap_or(&entry.path));
    if let Some(parent) = entry_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut entry_file = fs::File::create(&entry_path)?;

    let mut buf = vec![0; entry.size as usize];
    wad.seek(SeekFrom::Start(entry.offset as u64))?;
    wad.read_exact(&mut buf)?;

    entry_file.write_all(&buf)?;
    log::info!("wrote entry to {}", entry_path.display());

    Ok(())
}
