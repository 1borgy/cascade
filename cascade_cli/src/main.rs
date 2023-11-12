#![windows_subsystem = "windows"]
#![feature(error_generic_member_access)]

use std::{env, io, path::PathBuf};

use cascade::config::get_cascade_dir;
use fern::colors::{Color, ColoredLevelConfig};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

mod cli;

fn main() -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new().info(Color::Green);

    // TODO: consolidate getting paths in some core module
    let mut log_path = get_cascade_dir().unwrap_or(PathBuf::from(
        env::current_dir().expect("could not access cwd"),
    ));

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
                // if this expect fails then i give up
                .chain(
                    fern::log_file(log_path).expect("could not log to file"),
                ),
        )
        .apply()?;

    cli::main()?;

    Ok(())
}
