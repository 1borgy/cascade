use std::{fmt::Debug, fs::File, io::Write, path::PathBuf};

use cascade_lut::{self as lut, Lut};
use cascade_thugpro as thugpro;
use clap::{Args, Parser, Subcommand};

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
    Dump {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,
    },
    Randomize {
        #[arg(long)]
        input_dir: PathBuf,

        #[arg(long)]
        output_dir: PathBuf,

        #[arg(long, short)]
        name: String,

        #[arg(long)]
        female: bool,
    },
    RandomizeBulk {
        #[arg(long)]
        input_dir: PathBuf,

        #[arg(long)]
        output_dir: PathBuf,

        #[arg(long, short)]
        number: usize,

        #[arg(long)]
        female: bool,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let App {
        global: _global,
        command,
    } = App::parse();

    match command {
        Command::Dump { input, output } => {
            let entry = thugpro::save::Entry::at_path(&input).unwrap();
            let save = thugpro::save::Save::read_from(&entry).unwrap();
            let lut = Lut {
                checksum: lut::Checksum::load().unwrap(),
                compress: thugpro::lut::load_compress().unwrap(),
            };
            let dump = thugpro::Dump::new(&save, &lut);

            let mut file = File::create(output).unwrap();
            let contents =
                ron::ser::to_string_pretty(&dump, ron::ser::PrettyConfig::new()).unwrap();
            file.write(contents.as_bytes()).unwrap();

            Ok(())
        }
        Command::Randomize {
            input_dir,
            output_dir,
            name,
            female,
        } => {
            let entries = thugpro::save::find_entries(input_dir).unwrap();
            thugpro::random::randomize(&entries, output_dir, name, female)?;

            Ok(())
        }
        Command::RandomizeBulk {
            input_dir,
            output_dir,
            number,
            female,
        } => {
            let entries = thugpro::save::find_entries(input_dir).unwrap();
            thugpro::random::randomize_bulk(&entries, output_dir, number, female)?;

            Ok(())
        }
    }
}
