use std::{
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    result,
};

use cascade_lut::{self as lut, Lut};
use cascade_qb as qb;
use cascade_thugpro as thugpro;
use clap::{Args, Parser, Subcommand};
use encoding_rs::WINDOWS_1252;
use rand::seq::IndexedRandom;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("required at least two saves, got {0:?}")]
    NotEnoughSaves(usize),
}

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

        #[arg(long, short)]
        name: String,

        #[arg(long)]
        output: PathBuf,

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
            let ska = thugpro::save::Ska::read_from(&entry).unwrap();
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
        Command::Randomize {
            saves_dir,
            female,
            name,
            output,
        } => {
            let entries = thugpro::save::find_entries(saves_dir).unwrap();
            let mut rng = rand::rng();

            let base_entry = entries.choose(&mut rng).unwrap().clone();
            let mut base_ska = thugpro::save::Ska::read_from(&base_entry)?;
            let base_cas = thugpro::Cas::try_from(base_ska.clone())?;

            let cases = entries
                .into_iter()
                .filter_map(|entry| thugpro::save::Ska::read_from(&entry).ok())
                .filter_map(|ska| thugpro::Cas::try_from(ska).ok())
                .filter(|ska| match ska.summary.is_male {
                    // Female cas
                    thugpro::cas::Item::Present(qb::Symbol {
                        value: qb::Value::ZeroInt,
                        ..
                    }) => female,
                    // Male cas
                    thugpro::cas::Item::Present(qb::Symbol {
                        value: qb::Value::U8(1),
                        ..
                    }) => !female,
                    _ => false,
                })
                .collect::<Vec<_>>();

            fn get_filename(cas: &thugpro::cas::Cas) -> String {
                match &cas.summary.filename {
                    thugpro::cas::Item::Present(qb::Symbol {
                        value: qb::Value::String(bytes),
                        ..
                    }) => {
                        let (value, _, _) = WINDOWS_1252.decode(&bytes);
                        value.to_string()
                    }
                    _ => "unknown".to_string(),
                }
            }

            let (filename_bytes, _, _) = WINDOWS_1252.encode(name.as_str());

            let hat_cas = cases.choose(&mut rng).unwrap();
            let hat_appearance = &hat_cas.data.custom_skater.custom.appearance;

            let acc_cas = cases.choose(&mut rng).unwrap();
            let acc_appearance = &acc_cas.data.custom_skater.custom.appearance;

            let shirt_cas = cases.choose(&mut rng).unwrap();
            let shirt_appearance = &shirt_cas.data.custom_skater.custom.appearance;

            let legs_cas = cases.choose(&mut rng).unwrap();
            let legs_appearance = &legs_cas.data.custom_skater.custom.appearance;

            let shoes_cas = cases.choose(&mut rng).unwrap();
            let shoes_appearance = &shoes_cas.data.custom_skater.custom.appearance;

            let board_cas = cases.choose(&mut rng).unwrap();
            let board_appearance = &board_cas.data.custom_skater.custom.appearance;

            println!("name: {}", name);
            println!("base: {}", get_filename(&base_cas));
            println!("hat: {}", get_filename(&hat_cas));
            println!("accessories: {}", get_filename(&acc_cas));
            println!("shirt: {}", get_filename(&shirt_cas));
            println!("legs: {}", get_filename(&legs_cas));
            println!("shoes: {}", get_filename(&shoes_cas));
            println!("board: {}", get_filename(&board_cas));

            let transform = thugpro::Cas {
                summary: thugpro::cas::Summary {
                    filename: thugpro::cas::Item::Present(qb::Symbol {
                        kind: qb::Kind::String,
                        id: thugpro::id::FILENAME,
                        value: qb::Value::String(filename_bytes.into()),
                    }),
                    ..Default::default()
                },
                data: thugpro::cas::Data {
                    custom_skater: thugpro::cas::CustomSkater {
                        custom: thugpro::cas::Custom {
                            appearance: thugpro::cas::Appearance {
                                skater_m_hair: hat_appearance.skater_m_hair.clone(),
                                skater_f_hair: hat_appearance.skater_f_hair.clone(),
                                skater_m_hat_hair: hat_appearance.skater_m_hat_hair.clone(),
                                skater_f_hat_hair: hat_appearance.skater_f_hat_hair.clone(),
                                hat: hat_appearance.hat.clone(),
                                hat_logo: hat_appearance.hat_logo.clone(),
                                bare_torso: shirt_appearance.bare_torso.clone(),
                                skater_m_torso: shirt_appearance.skater_m_torso.clone(),
                                skater_f_torso: shirt_appearance.skater_f_torso.clone(),
                                front_logo: shirt_appearance.front_logo.clone(),
                                back_logo: shirt_appearance.back_logo.clone(),
                                elbowpads: shirt_appearance.elbowpads.clone(),
                                eyes: acc_appearance.eyes.clone(),
                                glasses: acc_appearance.glasses.clone(),
                                skater_m_hands: acc_appearance.skater_m_hands.clone(),
                                skater_f_hands: acc_appearance.skater_f_hands.clone(),
                                accessory1: acc_appearance.accessory1.clone(),
                                accessory2: acc_appearance.accessory2.clone(),
                                accessory3: acc_appearance.accessory3.clone(),
                                ped_m_accessories: acc_appearance.ped_m_accessories.clone(),
                                ped_f_accessories: acc_appearance.ped_f_accessories.clone(),
                                skater_m_backpack: acc_appearance.skater_m_backpack.clone(),
                                skater_f_backpack: acc_appearance.skater_f_backpack.clone(),
                                skater_m_legs: legs_appearance.skater_m_legs.clone(),
                                skater_f_legs: legs_appearance.skater_f_legs.clone(),
                                skater_m_lower_legs: legs_appearance.skater_m_lower_legs.clone(),
                                skater_f_lower_legs: legs_appearance.skater_f_lower_legs.clone(),
                                shoes: shoes_appearance.shoes.clone(),
                                socks: shoes_appearance.socks.clone(),
                                shoe_laces: shoes_appearance.shoe_laces.clone(),
                                board: board_appearance.board.clone(),
                                deck_graphic: board_appearance.deck_graphic.clone(),
                                griptape: board_appearance.griptape.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                },
            };

            transform.modify(&mut base_ska)?;

            fs::File::create(&output)?;
            let output_entry = thugpro::save::Entry::at_path(&output)?;
            base_ska.write_to(&output_entry)?;

            Ok(())
        }
    }
}
