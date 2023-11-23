use std::{
    fs,
    path::{Path, PathBuf},
};

use time;

use super::error::ActionError;

pub fn backup<P: AsRef<Path>>(
    backup_dir: P,
    saves_dir: P,
) -> Result<(), ActionError> {
    let backup_dir = backup_dir.as_ref();
    let saves_dir = saves_dir.as_ref();

    let datetime = time::OffsetDateTime::now_local().unwrap_or({
        log::warn!("could not get local timezone; using utc");
        time::OffsetDateTime::now_utc()
    });

    let subdir_name = format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        datetime.year(),
        u8::from(datetime.month()),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second()
    );

    let mut subdir = PathBuf::from(backup_dir);
    subdir.push(subdir_name);

    fs::create_dir_all(&subdir)?;

    // TODO: use SaveFile here since it's lazy now
    for file in fs::read_dir(saves_dir)? {
        // why him so confused ??
        let file = file?;
        let file_path = file.path();

        if file.file_type()?.is_file() {
            if let Some(extension) = file.path().extension() {
                if extension == "SKA" {
                    // holy shit let it stop im so sorry
                    // just use .filter() or something u moron
                    if let Some(file_name) = file_path.file_name() {
                        let mut backup_file_path = subdir.clone();
                        backup_file_path.push(file_name);

                        log::info!(
                            "backing up {:?} to {:?}",
                            file_path,
                            backup_file_path
                        );

                        fs::copy(file_path, backup_file_path)?;
                    }
                }
            }
        }
    }

    Ok(())
}
