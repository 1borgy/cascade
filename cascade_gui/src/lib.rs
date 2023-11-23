#![feature(error_generic_member_access)]

use std::{env, io, path::PathBuf};

use error::CascadeGuiError;
use fern::colors::{Color, ColoredLevelConfig};
use gui::Cascade;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

mod about;
mod config;
pub mod error;
mod gui;
pub mod paths;
mod resources;
mod theming;
mod views;

// TODO: turn this into CascadeResult or something
pub fn run() -> Result<(), CascadeGuiError> {
    let colors = ColoredLevelConfig::new().info(Color::Green);

    let mut log_path =
        paths::get_cascade_dir().unwrap_or(PathBuf::from(env::current_dir()?));

    log_path.push("cascade.log");

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let time = OffsetDateTime::now_local()
                .unwrap_or(OffsetDateTime::now_utc())
                .format(&Rfc3339)
                .unwrap_or("<unknown time>".to_string());

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
                // shut up
                // TODO: should probably move this to cascade_gui
                .level_for("wgpu_core", log::LevelFilter::Warn)
                .level_for("wgpu_hal", log::LevelFilter::Warn)
                .chain(io::stdout())
                .chain(fern::log_file(log_path)?),
        )
        .apply()?;

    Ok(Cascade::start()?)
}
