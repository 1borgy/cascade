use std::{
    fs,
    io::Read,
    sync::atomic::{AtomicBool, Ordering},
};

use cascade::save::SaveFile;
use rayon::prelude::ParallelIterator;

mod common;

fn read_save_file_bytes(save: &SaveFile) -> Vec<u8> {
    let filepath = save.filepath();
    let mut file =
        fs::File::open(&filepath).expect("could not open file for reading");

    let mut bytes = vec![];
    file.read_to_end(&mut bytes).expect("could not read file");

    bytes
}

fn diff_save_files(input_save: &SaveFile, output_save: &SaveFile) -> bool {
    let input_bytes = read_save_file_bytes(&input_save);
    let output_bytes = read_save_file_bytes(&output_save);

    if input_bytes.len() == output_bytes.len() {
        let mut num_diff_bytes = 0;

        for (input_byte, output_byte) in
            input_bytes.iter().zip(output_bytes.iter())
        {
            if input_byte != output_byte {
                num_diff_bytes += 1;
            }
        }

        let passed = num_diff_bytes == 0;
        let status = match passed {
            true => "pass",
            false => "fail",
        };

        println!(
            "result for {}: {} ({} bytes different)",
            input_save.filename(),
            status,
            num_diff_bytes
        );

        passed
    } else {
        println!("result for {}: input size ({}) and output size ({}) are different!", input_save.filename(), input_bytes.len(), output_bytes.len());
        false
    }
}

fn round_trip_save_file(
    input_save_file: &SaveFile,
    output_save_file: &SaveFile,
) -> bool {
    let input_save_content = input_save_file
        .load_content()
        .expect("could not load input save");

    output_save_file
        .write_content(&input_save_content)
        .expect("could not write output save");

    diff_save_files(input_save_file, output_save_file)
}

#[test]
fn round_trip() {
    let collection = common::save_collection();
    let output_dir = common::output_dir();

    let all_passed = AtomicBool::new(true);

    collection.par_iter().for_each(|input_save_file| {
        let output_save_file = input_save_file.with_dir(&output_dir);

        let file_passed =
            round_trip_save_file(&input_save_file, &output_save_file);

        all_passed.fetch_and(file_passed, Ordering::SeqCst);
    });

    assert!(all_passed.load(Ordering::SeqCst))
}

// TODO: round trip test copying identical trickset
// TODO: round trip test changing filename of files
