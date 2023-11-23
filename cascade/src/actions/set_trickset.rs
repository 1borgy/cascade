use std::{fs, path::Path};

use super::error::ActionError;

pub fn set_trickset<P: AsRef<Path>>(
    trickset_path: P,
    selected_path: P,
) -> Result<(), ActionError> {
    log::info!(
        "setting trickset at {:?} to {:?}",
        trickset_path.as_ref(),
        selected_path.as_ref()
    );

    fs::copy(selected_path, trickset_path)?;

    Ok(())
}
