use std::{env, fs, path::PathBuf};

use cascade::save::SaveCollection;

pub fn output_dir() -> PathBuf {
    let temp_dir = env::temp_dir();

    let mut output_dir = PathBuf::from(temp_dir);
    output_dir.push("cascade");
    output_dir.push("tests");

    println!("output dir: {:?}", output_dir);

    fs::create_dir_all(&output_dir).expect("could not create temp output dir");

    output_dir
}

pub fn save_collection() -> SaveCollection {
    let cwd = env::current_dir().expect("could not get cwd");

    let mut saves_dir = PathBuf::from(cwd);
    saves_dir.push("..");
    saves_dir.push("resources");
    saves_dir.push("saves");

    SaveCollection::at_dir(&saves_dir).expect("could not find saves directory")
}
