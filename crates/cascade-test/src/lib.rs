use std::{
    env,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::anyhow;

pub fn entries_dir(subdir: impl AsRef<Path>) -> PathBuf {
    env::current_dir()
        .expect("could not get cwd")
        .join("..")
        .join("..")
        .join("assets")
        .join("saves")
        .join(subdir)
}

fn diff_bytes(input_bytes: &[u8], output_bytes: &[u8]) -> anyhow::Result<()> {
    if input_bytes.len() == output_bytes.len() {
        let mut num_diff_bytes = 0;

        for (input_byte, output_byte) in input_bytes.iter().zip(output_bytes.iter()) {
            if input_byte != output_byte {
                num_diff_bytes += 1;
            }
        }

        if num_diff_bytes == 0 {
            Ok(())
        } else {
            Err(anyhow!("{} bytes different", num_diff_bytes))
        }
    } else {
        Err(anyhow!(
            "input size ({}) does not match output size ({})",
            input_bytes.len(),
            output_bytes.len()
        ))
    }
}

fn round_trip_entry<Save, Cas>(entry: &cascade_core::Entry) -> anyhow::Result<()>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let mut reader = entry.reader()?;
    let mut input_bytes = Vec::new();
    reader.read_to_end(&mut input_bytes)?;
    let mut input_cursor = Cursor::new(&input_bytes);

    let mut save = Save::read(&mut input_cursor)?;

    let cas = Cas::parse(&save)?;
    cas.modify(&mut save)?;

    let mut output_bytes = Vec::new();
    let mut output_cursor = Cursor::new(&mut output_bytes);
    save.write(&mut output_cursor)?;

    diff_bytes(&input_bytes, &output_bytes).map_err(|err| anyhow!("round_trip_entry: {}", err))
}

fn test_entry<Save, Cas>(entry: &cascade_core::Entry) -> anyhow::Result<()>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let mut errors = Vec::new();

    for result in [round_trip_entry::<Save, Cas>(entry)] {
        match result {
            Ok(_) => {}
            Err(err) => errors.push(err),
        }
    }

    if !errors.is_empty() {
        let failures = errors
            .into_iter()
            .map(|e| format!("  {}", e))
            .collect::<Vec<_>>()
            .join("\r\n");

        Err(anyhow!("[{}]\n{}", entry.name(), failures))
    } else {
        Ok(())
    }
}

pub async fn test_entries<Save, Cas>(entries: impl IntoIterator<Item = cascade_core::Entry>)
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let mut handles = Vec::new();
    for entry in entries.into_iter() {
        handles.push(tokio::task::spawn_blocking(move || {
            test_entry::<Save, Cas>(&entry)
        }))
    }

    let mut errors = Vec::new();
    for handle in handles {
        let result = handle
            .await
            .expect("unexpected join error collecting test results");

        match result {
            Ok(_) => {}
            Err(err) => errors.push(err),
        }
    }

    assert!(
        errors.is_empty(),
        "\n{}\n",
        errors
            .into_iter()
            .map(|e| format!("{}", e))
            .collect::<Vec<_>>()
            .join("\n\n")
    )
}
