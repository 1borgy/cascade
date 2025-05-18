use std::{fmt::Debug, fs::File, io::Write, path::PathBuf, result};

use cascade_lut::{self as lut, Lut};
use cascade_thugpro as thugpro;
use clap::{Args, Parser, Subcommand};
use rand::seq::IndexedRandom;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("required at least two saves, got {0:?}")]
    NotEnoughSaves(usize),
}

type Result<T, E = Error> = result::Result<T, E>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct App {
    #[clap(flatten)]
    global: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    GenerateLut {
        #[arg(long)]
        checksum: PathBuf,

        #[arg(long)]
        compressed_8: PathBuf,

        #[arg(long)]
        compressed_16: PathBuf,
    },
    Dump {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,
    },
    Randomize {
        #[arg(long)]
        saves_dir: PathBuf,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {}

fn main() -> Result<()> {
    let App {
        global: _global,
        command,
    } = App::parse();

    match command {
        Command::GenerateLut {
            checksum,
            compressed_8,
            compressed_16,
        } => {
            let checksum =
                lut::Checksum(serde_json::from_reader(File::open(checksum).unwrap()).unwrap());

            let mut file = File::create(".local/checksum.ron").unwrap();
            let contents = ron::to_string(&checksum).unwrap();
            file.write(&contents.as_bytes()).unwrap();
            Ok(())
        }
        Command::Dump { input, output } => {
            let entry = thugpro::save::Entry::at_path(&input).unwrap();
            let ska = thugpro::Ska::read_from(&entry).unwrap();
            let lut = Lut {
                checksum: lut::Checksum::load().unwrap(),
                compress: thugpro::lut::load_compress().unwrap(),
            };
            let dump = thugpro::Dump::new(&ska, &lut);

            let mut file = File::create(output).unwrap();
            let contents =
                ron::ser::to_string_pretty(&dump, ron::ser::PrettyConfig::new()).unwrap();
            file.write(contents.as_bytes()).unwrap();
            Ok(())
        }
        Command::Randomize { saves_dir } => {
            let entries = thugpro::save::find_entries(saves_dir).unwrap();
            let mut rng = rand::rng();
            let choices = entries
                .choose_multiple(&mut rng, 2)
                .map(|p| p.filepath())
                .collect::<Vec<_>>();
            println!("choices: {:?}", choices);
            Ok(())
        }
    }
}
