use std::{
    fmt::Display,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use cascade::{
    actions::{backup, ActionError},
    mutations::{self, Mutation},
    save::{SaveCollection, SaveFile, SaveFileExtension},
};
use rayon::prelude::ParallelIterator;

fn mutate_save(
    trickset_mutation: &mutations::Trickset,
    save: &SaveFile,
) -> Result<(), ActionError> {
    log::info!("copying trickset to save \"{}\"", save.filename());

    let content = save.load_content()?;
    let mutated_content = trickset_mutation.mutate(content)?;
    save.write_content(&mutated_content)?;

    log::info!(
        "successfully copied trickset to save \"{}\"",
        save.filename()
    );

    Ok(())
}

pub fn copy_trickset<P: AsRef<Path> + Sync>(
    trickset_path: P,
    backup_dir: P,
    saves_dir: P,
) -> Result<(usize, usize), ActionError> {
    backup(&backup_dir, &saves_dir)?;

    let trickset_file = SaveFile::at_path(&trickset_path)?;

    log::info!(
        "generating trickset mutation from \"{}\"",
        trickset_file.filename()
    );
    let trickset_mutation =
        mutations::Trickset::from_save(&trickset_file.load_content()?)?;

    let num_successful = Arc::new(AtomicUsize::new(0));

    let saves = SaveCollection::at_dir(&saves_dir)?
        .filter_extension(SaveFileExtension::SKA);

    let num_all_saves = saves.len();

    saves.par_iter().for_each(|save| {
        match mutate_save(&trickset_mutation, save) {
            Ok(_) => {
                num_successful.fetch_add(1, Ordering::SeqCst);
            }
            Err(err) => {
                log::warn!("error mutating save: {}", err);
            }
        }
    });

    for save in saves.iter() {
        save.overwrite_metadata()?;
    }

    Ok((num_successful.load(Ordering::SeqCst), num_all_saves))
}
