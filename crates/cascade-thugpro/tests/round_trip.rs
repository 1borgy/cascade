use std::{
    fs,
    io::Read,
    sync::atomic::{AtomicBool, Ordering},
};

use cascade_save as save;
use cascade_thugpro as thugpro;
mod common;

fn read_entry_bytes(save: &thugpro::Entry) -> Vec<u8> {
    let filepath = save.filepath();
    let mut file = fs::File::open(&filepath).expect("could not open file for reading");

    let mut bytes = vec![];
    file.read_to_end(&mut bytes).expect("could not read file");

    bytes
}

fn diff_save_files(input_entry: &thugpro::Entry, output_entry: &thugpro::Entry) -> bool {
    let input_bytes = read_entry_bytes(&input_entry);
    let output_bytes = read_entry_bytes(&output_entry);

    if input_bytes.len() == output_bytes.len() {
        let mut num_diff_bytes = 0;

        for (input_byte, output_byte) in input_bytes.iter().zip(output_bytes.iter()) {
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
            input_entry.filename(),
            status,
            num_diff_bytes
        );

        passed
    } else {
        println!(
            "result for {}: input size ({}) and output size ({}) are different!",
            input_entry.filename(),
            input_bytes.len(),
            output_bytes.len()
        );
        false
    }
}

fn round_trip_save_file(input_entry: &thugpro::Entry, output_entry: &thugpro::Entry) -> bool {
    let reader = input_entry.reader().unwrap();

    let input_save = thugpro::Save::read_from(input_entry).expect("could not load input save");

    input_save
        .write_to(output_entry)
        .expect("could not write output save");

    diff_save_files(input_entry, output_entry)
}

#[test]
fn round_trip() {
    let entries = common::entries();
    let output_dir = common::output_dir();

    let all_passed = AtomicBool::new(true);

    for entry in entries {
        let output_entry = entry.with_dir(&output_dir);

        let file_passed = round_trip_save_file(&entry, &output_entry);

        all_passed.fetch_and(file_passed, Ordering::SeqCst);
    }

    assert!(all_passed.load(Ordering::SeqCst))
}

// TODO: round trip test copying identical trickset
// TODO: round trip test changing filename of files
