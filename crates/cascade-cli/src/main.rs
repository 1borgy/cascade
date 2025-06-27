use std::{
    fmt::Debug,
    fs::{self, File},
    io::{BufReader, Write},
    path::PathBuf,
};

use cascade_dump as dump;
use cascade_lut::{self as lut, Lut};
use cascade_save::Save;
use cascade_thugpro as thugpro;
use cascade_wad::{hed, wad};
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
    Hed {
        #[arg(long)]
        input: PathBuf,
    },
    Wad {
        #[arg(long)]
        hed: PathBuf,

        #[arg(long)]
        wad: PathBuf,

        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {}

fn main() -> color_eyre::Result<()> {
    let App {
        global: _global,
        command,
    } = App::parse();

    color_eyre::install()?;
    env_logger::init();

    match command {
        Command::Dump { input, output } => {
            let entry = thugpro::Entry::at_path(&input)?;
            let save = Save::read(&mut entry.reader()?)?;
            let lut = Lut {
                checksum: lut::Checksum::load()?,
                compress: thugpro::lut::load_compress()?,
            };
            let dump = dump::Save::new(&save, &lut);

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
            let entries = thugpro::entry::find_entries(input_dir).unwrap();
            thugpro::random::randomize(&entries, output_dir, name, female)?;

            Ok(())
        }
        Command::RandomizeBulk {
            input_dir,
            output_dir,
            number,
            female,
        } => {
            let entries = thugpro::entry::find_entries(input_dir).unwrap();
            thugpro::random::randomize_bulk(&entries, output_dir, number, female)?;

            Ok(())
        }
        Command::Hed { input } => {
            let file = fs::File::open(input)?;
            let mut reader = BufReader::new(file);

            let file = hed::File::read(&mut reader)?;
            for entry in file.entries {
                println!("{:?}", entry);
            }

            Ok(())
        }
        Command::Wad { hed, wad, output } => {
            let hed = hed::File::read(&mut BufReader::new(fs::File::open(hed)?))?;
            let mut wad = fs::File::open(wad)?;

            wad::extract(hed, &mut wad, output)?;

            Ok(())
        }
    }
}
