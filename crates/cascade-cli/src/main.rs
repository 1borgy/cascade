use std::{
    fmt::Debug,
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::PathBuf,
    time::Duration,
};

use cascade_dump as dump;
use cascade_lut::{self as lut, Lut};
use cascade_save::Save;
use cascade_thps4 as thps4;
use cascade_thug as thug;
use cascade_thugpro as thugpro;
use cascade_wad::{hed, wad};
use clap::{Args, Parser, Subcommand};
use color_eyre::install;
use serde::Serialize;
use suppaftp::FtpStream;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct App {
    #[clap(flatten)]
    global: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Game {
    Thps4,
    Thug,
    #[default]
    Thugpro,
}

#[derive(Debug, Subcommand)]
enum Command {
    Dump {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long)]
        game: Game,
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
    Ftp {
        #[arg(long)]
        host: String,
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
        Command::Dump {
            input,
            output,
            game,
        } => {
            let entry = thugpro::Entry::at_path(&input)?;
            let save = Save::read(&mut entry.reader()?)?;
            let lut = Lut {
                checksum: lut::Checksum::load()?,
                compress: match game {
                    Game::Thps4 => thps4::lut::load_compress()?,
                    Game::Thug => thug::lut::load_compress()?,
                    Game::Thugpro => thugpro::lut::load_compress()?,
                },
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
        Command::Ftp { host } => {
            fn list(ftp_stream: &mut FtpStream) -> color_eyre::Result<Vec<suppaftp::list::File>> {
                let output = ftp_stream.list(None)?;
                log::info!("{}", output.join("\n"));

                let files: Vec<suppaftp::list::File> = output
                    .iter()
                    .filter_map(|line| suppaftp::list::File::try_from(line.as_str()).ok())
                    .collect();

                for file in files.iter() {
                    log::info!("{:?}", file);
                }

                Ok(files)
            }

            let game_id = "BASLUS-20731";
            let mc_path = "/mc/0";
            let mut ftp_stream = FtpStream::connect(host)?;
            ftp_stream.login("anonymous", "")?;
            ftp_stream.cwd(mc_path)?;

            let files = list(&mut ftp_stream)?;
            for file in files.iter().filter(|file| file.name().starts_with(game_id)) {
                ftp_stream.cwd(format!("{}/{}", mc_path, file.name()))?;
                list(&mut ftp_stream)?;
            }

            Ok(())
        }
    }
}
