use std::path::PathBuf;

pub fn find_entries(dir: &PathBuf) -> Vec<cascade_core::Entry> {
    match dir.read_dir() {
        Ok(dir) => {
            log::info!("finding entries in {:?}", dir);
            dir.filter_map(|file| file.ok())
                .filter_map(|file| {
                    let filepath = file.path();
                    log::info!("found file at {:?}", filepath);

                    // oops i should probably make this cleaner
                    if let Some(filename) = filepath.file_name() {
                        if filename.to_string_lossy().ends_with(".SKA") {
                            match cascade_core::Entry::create(&filepath) {
                                Ok(save) => {
                                    log::info!("found entry {:?}", filepath);
                                    Some(save)
                                }
                                Err(e) => {
                                    log::warn!("error loading entry {:?}: {}", filepath, e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(err) => {
            log::error!("error reading directory {:?}: {}", dir, err);
            Vec::new()
        }
    }
}
