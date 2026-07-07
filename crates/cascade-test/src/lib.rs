use std::{
    env,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::anyhow;

fn diff_bytes(input_bytes: &Vec<u8>, output_bytes: &Vec<u8>) -> anyhow::Result<()> {
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

fn round_trip_entry<Save>(entry: &cascade_core::Entry) -> anyhow::Result<()>
where
    Save: cascade_core::Save,
{
    let mut reader = entry.reader()?;
    let mut input_bytes = Vec::new();
    reader.read_to_end(&mut input_bytes)?;
    let mut input_cursor = Cursor::new(&input_bytes);

    let save = Save::read(&mut input_cursor)?;

    let mut output_bytes = Vec::new();
    let mut output_cursor = Cursor::new(&mut output_bytes);
    save.write(&mut output_cursor)?;

    diff_bytes(&input_bytes, &output_bytes).map_err(|err| anyhow!("round_trip_entry: {}", err))
}

fn test_entry<Save>(entry: &cascade_core::Entry) -> anyhow::Result<()>
where
    Save: cascade_core::Save,
{
    let mut errors = Vec::new();

    for result in vec![round_trip_entry::<Save>(entry)] {
        match result {
            Ok(_) => {}
            Err(err) => errors.push(err),
        }
    }

    if errors.len() > 0 {
        let failures = errors
            .into_iter()
            .map(|e| format!("  {}", e))
            .collect::<Vec<_>>()
            .join("\r\n");

        Err(anyhow!("{}\n{}", entry.name(), failures))
    } else {
        Ok(())
    }
}

pub async fn test_entries<Save>(entries: impl IntoIterator<Item = cascade_core::Entry>)
where
    Save: cascade_core::Save,
{
    let mut handles = Vec::new();
    for entry in entries.into_iter() {
        handles.push(tokio::task::spawn_blocking(move || {
            test_entry::<Save>(&entry)
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

    if errors.len() > 0 {
        let message = errors
            .into_iter()
            .map(|e| format!("{}", e))
            .collect::<Vec<_>>()
            .join("\n\n");

        println!("{}", message)
    }
}

pub fn entries_dir(subdir: impl AsRef<Path>) -> PathBuf {
    env::current_dir()
        .expect("could not get cwd")
        .join("..")
        .join("..")
        .join("assets")
        .join("saves")
        .join(subdir)
}
