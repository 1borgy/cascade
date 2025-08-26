use std::{
    fmt::Debug,
    fs,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use cascade_core as core;
#[cfg(feature = "dump")]
use cascade_dump as dump;
#[cfg(feature = "dump")]
use cascade_lut::{self as lut, Lut};
use cascade_thaw as thaw;
use cascade_thug2 as thug2;
#[cfg(feature = "wad")]
use cascade_wad::{hed, wad};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
#[cfg(feature = "ftp")]
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
    Thug2,
    Thaw,
    Rethawed,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[cfg(feature = "dump")]
    Dump {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long)]
        game: Game,
    },
    RoundTrip {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long)]
        game: Game,
    },
    Modify {
        #[arg(long)]
        from: PathBuf,

        #[arg(long)]
        to: PathBuf,

        #[arg(long)]
        game: Game,

        #[arg(long)]
        scales: bool,

        #[arg(long)]
        trickset: bool,
    },
    ModifyBulk {
        #[arg(long)]
        from: PathBuf,

        #[arg(long)]
        to_dir: PathBuf,

        #[arg(long)]
        game: Game,

        #[arg(long)]
        scales: bool,

        #[arg(long)]
        trickset: bool,
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
    #[cfg(feature = "wad")]
    Hed {
        #[arg(long)]
        input: PathBuf,
    },
    #[cfg(feature = "wad")]
    Wad {
        #[arg(long)]
        hed: PathBuf,

        #[arg(long)]
        wad: PathBuf,

        #[arg(long)]
        output: PathBuf,
    },
    #[cfg(feature = "ftp")]
    Ftp {
        #[arg(long)]
        host: String,
    },
}

#[cfg(feature = "dump")]
#[derive(serde::Serialize, serde::Deserialize)]
enum Dump {
    Neversoft(dump::Save),
    Rethawed(thaw::dump::Save),
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
        #[cfg(feature = "dump")]
        Command::Dump {
            input,
            output,
            game,
        } => {
            let entry = thug2::Entry::create(&input)?;
            let lut = Lut {
                checksum: lut::Checksum::load()?,
                compress: match game {
                    Game::Thps4 => thps4::lut::load_compress()?,
                    Game::Thug => thug::lut::load_compress()?,
                    Game::Thug2 => thug2::lut::load_compress()?,
                    Game::Thaw => thaw::lut::load_compress()?,
                    Game::Rethawed => thaw::lut::load_compress()?,
                },
            };
            let dump = match game {
                Game::Thps4 | Game::Thug | Game::Thug2 | Game::Thaw => {
                    let save = Save::read(&mut entry.reader()?)?;
                    Dump::Neversoft(dump::Save::new(&save, &lut))
                }
                Game::Rethawed => {
                    let save = thaw::save::Save::read(&mut entry.reader()?)?;
                    Dump::Rethawed(thaw::dump::Save::new(&save, &lut))
                }
            };

            let mut file = File::create(output).unwrap();
            let contents =
                ron::ser::to_string_pretty(&dump, ron::ser::PrettyConfig::new()).unwrap();
            file.write(contents.as_bytes()).unwrap();

            Ok(())
        }
        Command::RoundTrip {
            input,
            output,
            game,
        } => {
            fn round_trip<S, T, E>(
                input: &PathBuf,
                output: &PathBuf,
                parser: impl core::Parser<S, T, E>,
            ) -> Result<(), E>
            where
                E: From<io::Error>,
            {
                let file = fs::File::open(&input)?;
                let mut reader = BufReader::new(file);
                let mut save = parser.read(&mut reader)?;

                let parsed = parser.parse(&save)?;
                parser.modify(&mut save, &parsed)?;

                let file = fs::File::create(&output)?;
                let mut writer = BufWriter::new(file);
                parser.write(&save, &mut writer)?;

                Ok(())
            }

            match game {
                Game::Thug2 => round_trip(&input, &output, thug2::core::Parser {})?,
                Game::Rethawed => round_trip(&input, &output, thaw::core::Parser {})?,
                Game::Thps4 => todo!(),
                Game::Thug => todo!(),
                Game::Thaw => todo!(),
            }

            Ok(())
        }
        Command::Modify {
            from,
            to,
            game,
            scales,
            trickset,
        } => {
            let flags = core::Flags {
                trickset,
                scales,
                summary: false,
            };

            match game {
                Game::Thug2 => modify(&from, &to, thug2::core::Parser {}, flags)?,
                Game::Rethawed => modify(&from, &to, thaw::core::Parser {}, flags)?,
                Game::Thps4 => todo!(),
                Game::Thug => todo!(),
                Game::Thaw => todo!(),
            }

            Ok(())
        }
        Command::ModifyBulk {
            from,
            to_dir,
            game,
            scales,
            trickset,
        } => {
            let flags = core::Flags {
                trickset,
                scales,
                summary: false,
            };

            match game {
                Game::Thps4 => todo!(),
                Game::Thug => todo!(),
                Game::Thug2 => modify_bulk(
                    &from,
                    thug2::core::Explorer::new(to_dir),
                    thug2::core::Parser {},
                    flags,
                )?,
                Game::Thaw => todo!(),
                Game::Rethawed => modify_bulk(
                    &from,
                    thaw::core::Explorer::new(to_dir),
                    thaw::core::Parser {},
                    flags,
                )?,
            }

            Ok(())
        }
        Command::Randomize {
            input_dir,
            output_dir,
            name,
            female,
        } => {
            let entries = thug2::entry::find_entries(input_dir).unwrap();
            thug2::random::randomize(&entries, output_dir, name, female)?;

            Ok(())
        }
        Command::RandomizeBulk {
            input_dir,
            output_dir,
            number,
            female,
        } => {
            let entries = thug2::entry::find_entries(input_dir).unwrap();
            thug2::random::randomize_bulk(&entries, output_dir, number, female)?;

            Ok(())
        }
        #[cfg(feature = "wad")]
        Command::Hed { input } => {
            let file = fs::File::open(input)?;
            let mut reader = BufReader::new(file);

            let file = hed::File::read(&mut reader)?;
            for entry in file.entries {
                println!("{:?}", entry);
            }

            Ok(())
        }
        #[cfg(feature = "wad")]
        Command::Wad { hed, wad, output } => {
            let hed = hed::File::read(&mut BufReader::new(fs::File::open(hed)?))?;
            let mut wad = fs::File::open(wad)?;

            wad::extract(hed, &mut wad, output)?;

            Ok(())
        }
        #[cfg(feature = "ftp")]
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

fn modify<S, T, E>(
    from: &PathBuf,
    to: &PathBuf,
    parser: impl core::Parser<S, T, E>,
    flags: core::Flags,
) -> Result<(), E>
where
    E: From<io::Error>,
{
    let from_file = fs::File::open(&from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = parser.read(&mut from_reader)?;

    let to_file = fs::File::open(&to)?;
    let mut to_reader = BufReader::new(to_file);
    let mut to_save = parser.read(&mut to_reader)?;

    let from_parsed = parser.parse(&from_save)?;
    let from_parsed = parser.mask(from_parsed, flags);
    parser.modify(&mut to_save, &from_parsed)?;

    let to_file = fs::File::create(&to)?;
    let mut writer = BufWriter::new(to_file);
    parser.write(&to_save, &mut writer)?;

    Ok(())
}

fn modify_bulk<Entry, Save, Cas, Error>(
    from: &PathBuf,
    explorer: impl core::Explorer<Entry, Error>,
    parser: impl core::Parser<Save, Cas, Error>,
    flags: core::Flags,
) -> Result<(), Error>
where
    Error: From<io::Error> + Debug,
{
    fn modify_one<Entry, Save, Cas, Error>(
        entry: &Entry,
        transform: &Cas,
        explorer: &impl core::Explorer<Entry, Error>,
        parser: &impl core::Parser<Save, Cas, Error>,
    ) -> Result<(), Error> {
        let mut reader = explorer.reader(&entry)?;
        let mut save = parser.read(&mut reader)?;
        parser.modify(&mut save, &transform)?;
        let mut writer = explorer.writer(&entry)?;
        parser.write(&save, &mut writer)?;
        Ok(())
    }

    let from_file = fs::File::open(&from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = parser.read(&mut from_reader)?;
    let from_parsed = parser.parse(&from_save)?;
    let transform = parser.mask(from_parsed, flags);

    for entry in explorer.list()? {
        let name = explorer.name(&entry);
        match modify_one(&entry, &transform, &explorer, &parser) {
            Ok(_) => {
                println!("successfully copied {}", name)
            }
            Err(e) => {
                println!("error copying {}: {:?}", name, e)
            }
        }
    }

    Ok(())
}
