#[cfg(feature = "dump")]
use std::io::Write;
use std::{
    fmt::Debug,
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

#[cfg(feature = "dump")]
use cascade_core::Save;
#[cfg(feature = "dump")]
use cascade_dump as dump;
#[cfg(feature = "dump")]
use cascade_lut::{self as lut, Lut};
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
    Thps3,
    Thps4,
    Thug,
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
    RoundTrip {
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
    Crc {
        #[arg(long)]
        value: String,
    },
}

#[cfg(feature = "dump")]
#[derive(serde::Serialize, serde::Deserialize)]
enum Dump {
    Thps3(cascade_thps3::dump::Save),
    Neversoft(dump::Save),
    Thaw(cascade_thaw::dump::Save),
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
            let entry = cascade_core::Entry::create(&input)?;
            let lut = Lut {
                checksum: lut::Checksum::load()?,
                compress: match game {
                    Game::Thps3 => cascade_thps3::lut::load_compress()?,
                    Game::Thps4 => cascade_thps4::lut::load_compress()?,
                    Game::Thug => cascade_thug::lut::load_compress()?,
                    Game::Thug2 => cascade_thug2::lut::load_compress()?,
                    Game::Thaw => cascade_thaw::lut::load_compress()?,
                },
            };
            let dump = match game {
                Game::Thps3 => {
                    let save = cascade_thps3::Save::read(&mut entry.reader()?)?;
                    Dump::Thps3(cascade_thps3::dump::Save::new(&save, &lut))
                }
                Game::Thps4 | Game::Thug | Game::Thug2 => {
                    let save = cascade_save::Save::read(&mut entry.reader()?)?;
                    Dump::Neversoft(dump::Save::new(&save, &lut))
                }
                Game::Thaw => {
                    let save = cascade_thaw::Save::read(&mut entry.reader()?)?;
                    Dump::Thaw(cascade_thaw::dump::Save::new(&save, &lut))
                }
            };

            let mut file = fs::File::create(output).unwrap();
            let contents =
                ron::ser::to_string_pretty(&dump, ron::ser::PrettyConfig::new()).unwrap();

            file.write_all(contents.as_bytes()).unwrap();

            Ok(())
        }
        Command::RoundTrip {
            input,
            output,
            game,
        } => {
            match game {
                Game::Thps3 => {
                    round_trip::<cascade_thps3::Save, cascade_thps3::Cas>(&input, &output)?
                }
                Game::Thps4 => {
                    round_trip::<cascade_thps4::Save, cascade_thps4::Cas>(&input, &output)?
                }
                Game::Thug => round_trip::<cascade_thug::Save, cascade_thug::Cas>(&input, &output)?,
                Game::Thug2 => {
                    round_trip::<cascade_thug2::Save, cascade_thug2::Cas>(&input, &output)?
                }
                Game::Thaw => round_trip::<cascade_thaw::Save, cascade_thaw::Cas>(&input, &output)?,
            };

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
                Game::Thps3 => {
                    modify::<cascade_thps3::Save, cascade_thps3::Cas>(&from, &to, flags)?
                }
                Game::Thps4 => {
                    modify::<cascade_thps4::Save, cascade_thps4::Cas>(&from, &to, flags)?
                }
                Game::Thug => modify::<cascade_thug::Save, cascade_thug::Cas>(&from, &to, flags)?,
                Game::Thug2 => {
                    modify::<cascade_thug2::Save, cascade_thug2::Cas>(&from, &to, flags)?
                }
                Game::Thaw => modify::<cascade_thaw::Save, cascade_thaw::Cas>(&from, &to, flags)?,
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
                Game::Thps3 => modify_bulk::<cascade_thps3::Save, cascade_thps3::Cas>(
                    &from,
                    cascade_thps3::find_entries(&to_dir),
                    flags,
                )?,
                Game::Thps4 => modify_bulk::<cascade_thps4::Save, cascade_thps4::Cas>(
                    &from,
                    cascade_thps4::find_entries(&to_dir),
                    flags,
                )?,
                Game::Thug => modify_bulk::<cascade_thug::Save, cascade_thug::Cas>(
                    &from,
                    cascade_thug::find_entries(&to_dir),
                    flags,
                )?,
                Game::Thug2 => modify_bulk::<cascade_thug2::Save, cascade_thug2::Cas>(
                    &from,
                    cascade_thug2::find_entries(&to_dir),
                    flags,
                )?,
                Game::Thaw => modify_bulk::<cascade_thaw::Save, cascade_thaw::Cas>(
                    &from,
                    cascade_thaw::find_entries(&to_dir),
                    flags,
                )?,
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
        Command::Crc { value } => {
            let checksum = cascade_crc::checksum(&value.as_bytes().to_vec());
            println!("{:#08x}", checksum);
            Ok(())
        },
    }
}

fn modify<Save, Cas>(
    from: &PathBuf,
    to: &PathBuf,
    flags: cascade_core::Flags,
) -> Result<(), cascade_core::Error>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let from_file = fs::File::open(from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = Save::read(&mut from_reader)?;

    let to_file = fs::File::open(to)?;
    let mut to_reader = BufReader::new(to_file);
    let mut to_save = Save::read(&mut to_reader)?;

    let from_parsed = Cas::parse(&from_save)?;
    let transform = from_parsed.mask(flags);
    transform.modify(&mut to_save)?;

    let to_file = fs::File::create(to)?;
    let mut writer = BufWriter::new(to_file);
    to_save.write(&mut writer)?;

    Ok(())
}

fn modify_bulk<Save, Cas>(
    from: &PathBuf,
    to_entries: Vec<cascade_core::Entry>,
    flags: cascade_core::Flags,
) -> cascade_core::Result<()>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    fn modify_one<Save, Cas>(
        entry: &cascade_core::Entry,
        transform: &Cas,
    ) -> cascade_core::Result<()>
    where
        Save: cascade_core::Save,
        Cas: cascade_core::Cas<Save = Save>,
    {
        let mut reader = entry.reader()?;
        let mut save = Save::read(&mut reader)?;

        transform.modify(&mut save)?;

        let mut writer = entry.writer()?;
        save.write(&mut writer)?;

        Ok(())
    }

    let from_file = fs::File::open(from)?;
    let mut from_reader = BufReader::new(from_file);
    let from_save = Save::read(&mut from_reader)?;
    let from_parsed = Cas::parse(&from_save)?;
    let transform = from_parsed.mask(flags);

    for entry in to_entries {
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

fn round_trip<Save, Cas>(from: &PathBuf, to: &PathBuf) -> Result<(), cascade_core::Error>
where
    Save: cascade_core::Save,
    Cas: cascade_core::Cas<Save = Save>,
{
    let from_file = fs::File::open(from)?;
    let mut reader = BufReader::new(from_file);
    let mut save = Save::read(&mut reader)?;
    let parsed = Cas::parse(&save)?;
    parsed.modify(&mut save)?;

    let to_file = fs::File::create(to)?;
    let mut writer = BufWriter::new(to_file);
    save.write(&mut writer)?;

    Ok(())
}
