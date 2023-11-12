// TODO: remove this file
use std::{
    fs::{File, Metadata},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use filetime;

use crate::{
    files,
    save::{SaveData, SaveError},
};

pub fn load_reader<P: AsRef<Path>>(path: P) -> io::Result<BufReader<File>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader)
}

pub fn load_writer<P: AsRef<Path>>(path: P) -> io::Result<BufWriter<File>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    Ok(writer)
}

pub fn load_save(filepath: &PathBuf) -> Result<SaveFile, SaveError> {
    // god forgive me
    let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();
    let save = SaveData::from_reader(&mut files::load_reader(filepath)?)?;
    let metadata = filepath.metadata()?;

    Ok(SaveFile {
        data: save,
        filename,
        metadata,
    })
}

pub fn write_save(
    filepath: &PathBuf,
    save_file: &SaveFile,
) -> Result<(), SaveError> {
    let file = File::create(&filepath)?;
    let mut writer = BufWriter::new(file);

    save_file.data.write(&mut writer)?;
    writer.flush()?;

    let original_mod_time =
        filetime::FileTime::from_last_modification_time(&save_file.metadata);

    log::info!("setting file modification time to {:?}", original_mod_time);
    filetime::set_file_mtime(&filepath, original_mod_time)?;

    log::info!("wrote save to {:?}", &filepath);
    Ok(())
}

pub struct SaveFile {
    pub data: SaveData,
    pub filename: String,
    pub metadata: Metadata,
}

impl SaveFile {
    pub fn with_data(&self, new_save: SaveData) -> Self {
        SaveFile {
            data: new_save,
            filename: self.filename.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

pub fn load_saves_from_dir(dir: &PathBuf) -> anyhow::Result<Vec<SaveFile>> {
    if !dir.is_dir() {
        return Err(anyhow!("invalid saves directory {:?}", dir));
    }

    let saves: Vec<SaveFile> = dir
        .read_dir()?
        .filter_map(|file| file.ok())
        .filter(|file| {
            file.path()
                .extension()
                .filter(|ext| ext.to_str() == Some("SKA"))
                .is_some()
        })
        .filter_map(|file| {
            let filepath = file.path();

            match files::load_save(&filepath) {
                Ok(save) => {
                    log::debug!("successfully loaded save {:?}", filepath);
                    Some(save)
                }
                Err(e) => {
                    log::debug!("error loading save {:?}: {}", filepath, e);
                    None
                }
            }
        })
        .collect();

    log::info!("loaded {} saves", saves.len());

    Ok(saves)
}

pub fn write_saves_to_dir(
    saves: &Vec<SaveFile>,
    dir: &PathBuf,
) -> Result<(), SaveError> {
    log::info!("writing save collection to {:?}", dir);

    for savefile in saves.iter() {
        let new_save_path = dir.join(Path::new(savefile.filename.as_str()));

        write_save(&new_save_path, savefile)?;
    }

    Ok(())
}

pub fn with_copied_tricksets(
    saves: &Vec<SaveFile>,
    tricksrc: &SaveFile,
) -> Vec<SaveFile> {
    log::info!("copying trickset to all saves");

    let mut new_saves = vec![];

    for save in saves.iter() {
        match save.data.with_copied_trickset(&tricksrc.data) {
            Ok(new_save) => new_saves.push(save.with_data(new_save)),
            Err(e) => log::error!(
                "could not copy trickset to save {:?}: {}",
                save.filename,
                e
            ),
        }
    }

    new_saves
}
