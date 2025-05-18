use std::{
    fs,
    path::{Path, PathBuf},
};

use cascade_qb as qb;
use encoding_rs::WINDOWS_1252;
use rand::seq::IndexedRandom;

use crate::{Cas, Result, cas, id, save};

pub fn randomize(
    entries: &Vec<save::Entry>,
    output_dir: impl AsRef<Path>,
    name: impl AsRef<str>,
    female: bool,
) -> Result<()> {
    let name = name.as_ref();
    let output_dir = output_dir.as_ref();

    let mut rng = rand::rng();

    let base_entry = entries.choose(&mut rng).unwrap().clone();
    let mut base_save = save::Save::read_from(&base_entry)?;
    let base_cas = Cas::try_from(base_save.clone())?;

    let cases = entries
        .into_iter()
        .filter_map(|entry| save::Save::read_from(&entry).ok())
        .filter_map(|save| Cas::try_from(save).ok())
        .filter(|save| match save.summary.is_male {
            // Female cas
            cas::Item::Present(qb::Symbol {
                value: qb::Value::ZeroInt,
                ..
            }) => female,
            // Male cas
            cas::Item::Present(qb::Symbol {
                value: qb::Value::U8(1),
                ..
            }) => !female,
            _ => false,
        })
        .collect::<Vec<_>>();

    fn get_filename(cas: &cas::Cas) -> String {
        match &cas.summary.filename {
            cas::Item::Present(qb::Symbol {
                value: qb::Value::String(bytes),
                ..
            }) => {
                let (value, _, _) = WINDOWS_1252.decode(&bytes);
                value.to_string()
            }
            _ => "unknown".to_string(),
        }
    }

    let (filename_bytes, _, _) = WINDOWS_1252.encode(name.as_ref());

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

    println!("");
    println!("name: {}", name);
    println!("base: {}", get_filename(&base_cas));
    println!("hat: {}", get_filename(&hat_cas));
    println!("accessories: {}", get_filename(&acc_cas));
    println!("shirt: {}", get_filename(&shirt_cas));
    println!("legs: {}", get_filename(&legs_cas));
    println!("shoes: {}", get_filename(&shoes_cas));
    println!("board: {}", get_filename(&board_cas));

    let transform = Cas {
        summary: cas::Summary {
            filename: cas::Item::Present(qb::Symbol {
                kind: qb::Kind::String,
                id: id::FILENAME,
                value: qb::Value::String(filename_bytes.into()),
            }),
            ..Default::default()
        },
        data: cas::Data {
            custom_skater: cas::CustomSkater {
                custom: cas::Custom {
                    appearance: cas::Appearance {
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
                        sleeves: shirt_appearance.shoe_laces.clone(),
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
                        kneepads: legs_appearance.skater_f_lower_legs.clone(),
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

    transform.modify(&mut base_save)?;

    fs::create_dir_all(output_dir)?;

    let output_path = PathBuf::from(output_dir).join(format!("{}.SKA", name));
    fs::File::create(&output_path)?;

    let output_entry = save::Entry::at_path(&output_path)?;
    base_save.write_to(&output_entry)?;

    Ok(())
}

pub fn randomize_bulk(
    entries: &Vec<save::Entry>,
    output_dir: impl AsRef<Path>,
    number: usize,
    female: bool,
) -> Result<()> {
    for i in 0..number {
        let name = format!("rand{}", i);
        randomize(entries, &output_dir, name, female)?;
    }

    Ok(())
}
