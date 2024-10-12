#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![feature(error_generic_member_access, path_add_extension)]

use std::{io, path::Path, result};

use app::Cascade;
use clap::Parser;
use config::{Config, Selections};
use fern::colors::{Color, ColoredLevelConfig};
use iced::{window, Size};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

mod app;
mod config;
mod fonts;
mod paths;
mod screen;
mod tasks;
mod theme;
mod widget;

const ICON_BYTES: &[u8] = include_bytes!("../../resources/cascade.ico");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an io error occurred: {0}")]
    Io(#[from] io::Error),

    #[error("an logging error occurred: {0}")]
    Log(#[from] log::SetLoggerError),

    #[error("a gui error occurred: {0}")]
    Gui(#[from] iced::Error),

    #[error("a path error occurred: {0}")]
    Paths(#[from] paths::Error),
}

type Result<T> = result::Result<T, Error>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn configure_logging(path: impl AsRef<Path>) -> Result<()> {
    let colors = ColoredLevelConfig::new().info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let time = OffsetDateTime::now_local()
                .unwrap_or(OffsetDateTime::now_utc())
                .format(&Rfc3339)
                .unwrap_or("<?>".to_string());

            out.finish(format_args!(
                "[{}] {} [{}] {}",
                time,
                record.target(),
                colors.color(record.level()),
                message,
            ))
        })
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .level_for("wgpu_core", log::LevelFilter::Warn)
                .level_for("wgpu_hal", log::LevelFilter::Warn)
                .chain(io::stdout())
                .chain(fern::log_file(&path)?),
        )
        .apply()?;

    log::info!("logging to {:?}", path.as_ref());

    Ok(())
}

fn main() -> Result<()> {
    let Args { debug } = Args::parse();

    let cascade_dir =
        paths::cascade_dir().expect("could not determine cascade dir");

    configure_logging(paths::log(&cascade_dir))?;

    let config_path = paths::config(&cascade_dir);
    let selections_path = paths::selections(&cascade_dir);

    let config = Config::load(&config_path).unwrap_or_default();
    log::info!("loaded config: {:?}", config);

    let selections = Selections::load(&selections_path).unwrap_or_default();
    log::info!("loaded selections: {:?}", selections);

    Ok(iced::application("cascade", Cascade::update, Cascade::view)
        .window(window::Settings {
            min_size: Some(Size::new(720., 520.)),
            icon: window::icon::from_file_data(
                ICON_BYTES,
                Some(image::ImageFormat::Ico),
            )
            .ok(),
            ..Default::default()
        })
        .font(fonts::IOSEVKA_REGULAR_BYTES)
        .font(fonts::IOSEVKA_BOLD_BYTES)
        .default_font(fonts::IOSEVKA_REGULAR)
        .scale_factor(Cascade::scale_factor)
        .theme(Cascade::theme)
        .run_with(move || {
            Cascade::new((cascade_dir, config, selections, debug))
        })?)
}
