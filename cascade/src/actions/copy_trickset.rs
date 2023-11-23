use std::{
    fmt::Display,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use rayon::prelude::ParallelIterator;

use crate::{
    actions::{backup::backup, error::ActionError},
    mutations::{self, Mutation},
    save::{SaveCollection, SaveFile, SaveFileExtension},
};

fn try_result<V, E>(result: Result<V, E>) -> Option<V>
where
    E: Display,
{
    match result {
        Err(err) => {
            log::info!("error: {}", err);
            None
        }
        Ok(v) => Some(v),
    }
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

    // TODO: move this to functions
    // TODO: please do it quickly
    // TODO: it's so messy
    // TODO:
    // TODO: please
    saves.par_iter().for_each(|save| {
        try_result(save.load_content()).and_then(|content| {
            match trickset_mutation.mutate(content) {
                Ok(new_content) => {
                    log::info!(
                        "successfully copied trickset to save \"{}\"",
                        save.filename()
                    );

                    try_result(
                        save.with_dir(&saves_dir).write_content(&new_content),
                    )
                    .and_then(|_| {
                        num_successful.fetch_add(1, Ordering::SeqCst);
                        None::<()>
                    });
                }
                Err(err) => {
                    log::error!(
                        "could not copy trickset to save \"{}\": {}",
                        save.filename(),
                        err
                    );
                }
            };

            None::<()>
        });
    });

    for save in saves.iter() {
        save.overwrite_metadata()?;
    }

    Ok((num_successful.load(Ordering::SeqCst), num_all_saves))
}
