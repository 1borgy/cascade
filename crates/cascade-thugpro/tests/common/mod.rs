use std::{env, fs, path::PathBuf};

use cascade_thugpro as thugpro;

pub fn output_dir() -> PathBuf {
    let temp_dir = env::temp_dir();

    let mut output_dir = PathBuf::from(temp_dir);
    output_dir.push("cascade");
    output_dir.push("tests");

    println!("output dir: {:?}", output_dir);

    fs::create_dir_all(&output_dir).expect("could not create temp output dir");

    output_dir
}

pub fn entries() -> Vec<thugpro::Entry> {
    let cwd = env::current_dir().expect("could not get cwd");

    let saves_dir = PathBuf::from(cwd)
        .join("..")
        .join("..")
        .join("assets")
        .join("saves");

    thugpro::save::find_entries(&saves_dir).expect("could not find saves directory")
}
