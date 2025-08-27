use std::{
    fmt::Debug,
    fs,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use cascade_core;
#[cfg(feature = "dump")]
use cascade_dump as dump;
#[cfg(feature = "dump")]
use cascade_lut::{self as lut, Lut};
use cascade_thaw as thaw;
use cascade_thug2 as thug2;
#[cfg(feature = "wad")]
use cascade_wad::{hed, wad};
use clap::{Parser, Subcommand};
use serde::Serialize;
#[cfg(feature = "ftp")]
use suppaftp::FtpStream;

/// A CLI-only version of cascade.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Game {
    #[default]
    Thug2,
    Thaw,
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
    /// Perform a single modification.
    Modify {
        /// Path to the save file to copy from.
        #[arg(long)]
        from: PathBuf,

        /// Path to the save file to copy to.
        #[arg(long)]
        to: PathBuf,

        /// Which game the saves are for.
        #[arg(long)]
        game: Game,

        /// If specified, copy scales.
        #[arg(long)]
        scales: bool,

        /// If specified, copy trickset.
        #[arg(long)]
        trickset: bool,
    },
    /// Perform a modification in bulk (i.e. all saves in a directory).
    ModifyBulk {
        /// Path to the save file to copy from.
        #[arg(long)]
        from: PathBuf,

        /// Path to the directory containing all save files to copy to.
        #[arg(long)]
        to_dir: PathBuf,

        /// Which game the saves are for.
        #[arg(long)]
        game: Game,

        /// If specified, copy scales.
        #[arg(long)]
        scales: bool,

        /// If specified, copy trickset.
        #[arg(long)]
        trickset: bool,
    },
    // Randomize {
    //     #[arg(long)]
    //     input_dir: PathBuf,
    //
    //     #[arg(long)]
    //     output_dir: PathBuf,
    //
    //     #[arg(long, short)]
    //     name: String,
    //
    //     #[arg(long)]
    //     female: bool,
    // },
    // RandomizeBulk {
    //     #[arg(long)]
    //     input_dir: PathBuf,
    //
    //     #[arg(long)]
    //     output_dir: PathBuf,
    //
    //     #[arg(long, short)]
    //     number: usize,
    //
    //     #[arg(long)]
    //     female: bool,
    // },
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

fn main() -> color_eyre::Result<()> {
    let App { command } = App::parse();

    color_eyre::install()?;
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    match command {
        #[cfg(feature = "dump")]
        Command::Dump {
            input,
            output,
            game,
        } => {
            use std::io::Write;

            let entry = thug2::Entry::create(&input)?;
            let lut = Lut {
                checksum: lut::Checksum::load()?,
                compress: match game {
                    Game::Thug2 => thug2::lut::load_compress()?,
                    Game::Thaw => thaw::lut::load_compress()?,
                },
            };
            let dump = match game {
                Game::Thug2 => {
                    let save = cascade_save::Save::read(&mut entry.reader()?)?;
                    Dump::Neversoft(dump::Save::new(&save, &lut))
                }
                Game::Thaw => {
                    let save = thaw::Save::read(&mut entry.reader()?)?;
                    Dump::Rethawed(thaw::dump::Save::new(&save, &lut))
                }
            };

            let mut file = fs::File::create(output).unwrap();
            let contents =
                ron::ser::to_string_pretty(&dump, ron::ser::PrettyConfig::new()).unwrap();
            file.write(contents.as_bytes()).unwrap();

            Ok(())
        }
        Command::Modify {
            from,
            to,
            game,
            scales,
            trickset,
        } => {
            let flags = cascade_core::Flags {
                trickset,
                scales,
                summary: false,
            };

            if !scales && !trickset {
                log::warn!("neither `--scales` nor `--trickset` are specified; expect no changes")
            }

            match game {
                Game::Thug2 => modify::<thug2::Save, thug2::Cas, thug2::Error>(&from, &to, flags)?,
                Game::Thaw => modify::<thaw::Save, thaw::Cas, thaw::Error>(&from, &to, flags)?,
            }

            log::info!("successfully copied to {}", to.display());

            Ok(())
        }
        Command::ModifyBulk {
            from,
            to_dir,
            game,
            scales,
            trickset,
        } => {
            let flags = cascade_core::Flags {
                trickset,
                scales,
                summary: false,
            };

            if !scales && !trickset {
                log::warn!("neither `--scales` nor `--trickset` are specified; expect no changes")
            }

            match game {
                Game::Thug2 => modify_bulk(&from, thug2::Core::new(to_dir), flags)?,
                Game::Thaw => modify_bulk(&from, thaw::Core::new(to_dir), flags)?,
            }

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

fn modify<Save, Cas, Error>(
    from: &PathBuf,
    to: &PathBuf,
    flags: cascade_core::Flags,
) -> Result<(), Error>
where
    Save: cascade_core::Save<Error = Error>,
    Cas: cascade_core::Cas<Save = Save, Error = Error>,
    Error: From<io::Error> + Debug,
{
    let from_file = fs::File::open(&from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = Save::read(&mut from_reader)?;

    let to_file = fs::File::open(&to)?;
    let mut to_reader = BufReader::new(to_file);
    let mut to_save = Save::read(&mut to_reader)?;

    let from_parsed = Cas::parse(&from_save)?;
    let transform = from_parsed.mask(flags);
    transform.modify(&mut to_save)?;

    let to_file = fs::File::create(&to)?;
    let mut writer = BufWriter::new(to_file);
    to_save.write(&mut writer)?;

    Ok(())
}

fn modify_bulk<Entry, Save, Cas, Error>(
    from: &PathBuf,
    core: impl cascade_core::Core<Entry = Entry, Save = Save, Cas = Cas, Error = Error>,
    flags: cascade_core::Flags,
) -> Result<(), Error>
where
    Entry: cascade_core::Entry<Error = Error>,
    Save: cascade_core::Save<Error = Error>,
    Cas: cascade_core::Cas<Save = Save, Error = Error>,
    Error: From<io::Error> + Debug,
{
    fn modify_one<Entry, Save, Cas, Error>(entry: &Entry, transform: &Cas) -> Result<(), Error>
    where
        Entry: cascade_core::Entry<Error = Error>,
        Save: cascade_core::Save<Error = Error>,
        Cas: cascade_core::Cas<Save = Save, Error = Error>,
        Error: From<io::Error> + Debug,
    {
        let mut reader = entry.reader()?;
        let mut save = Save::read(&mut reader)?;

        transform.modify(&mut save)?;

        let mut writer = entry.writer()?;
        save.write(&mut writer)?;

        Ok(())
    }

    let from_file = fs::File::open(&from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = Save::read(&mut from_reader)?;
    let from_parsed = Cas::parse(&from_save)?;
    let transform = from_parsed.mask(flags);

    for entry in core.list_entries()? {
        let name = entry.name();
        match modify_one(&entry, &transform) {
            Ok(_) => {
                log::info!("successfully copied to {}", name)
            }
            Err(e) => {
                log::info!("error copying to {}: {:?}", name, e)
            }
        }
    }

    Ok(())
}
