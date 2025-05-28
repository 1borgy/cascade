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
mod dashboard;
mod fonts;
mod paths;
mod tasks;
mod theme;
mod widget;

pub use config::Theme;

pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
pub type Content<'a, Message> = iced::widget::pane_grid::Content<'a, Message, Theme, Renderer>;
pub type TitleBar<'a, Message> = iced::widget::pane_grid::TitleBar<'a, Message, Theme, Renderer>;
pub type Column<'a, Message> = iced::widget::Column<'a, Message, Theme, Renderer>;
pub type Row<'a, Message> = iced::widget::Row<'a, Message, Theme, Renderer>;
pub type Text<'a> = iced::widget::Text<'a, Theme, Renderer>;
pub type Container<'a, Message> = iced::widget::Container<'a, Message, Theme, Renderer>;
pub type Button<'a, Message> = iced::widget::Button<'a, Message, Theme>;

const CASCADE_ICON_BYTES: &[u8] = include_bytes!("../../../assets/cascade.ico");

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
                .level_for("wgpu_core", log::LevelFilter::Error)
                .level_for("wgpu_hal", log::LevelFilter::Error)
                .level_for("iced_winit", log::LevelFilter::Error)
                .chain(io::stdout())
                .chain(fern::log_file(&path)?),
        )
        .apply()?;

    log::info!("logging to {:?}", path.as_ref());

    Ok(())
}

fn main() -> Result<()> {
    let Args { debug } = Args::parse();

    let cascade_dir = paths::cascade_dir().expect("could not determine cascade dir");

    configure_logging(paths::log(&cascade_dir))?;

    let config = Config::load(paths::config(&cascade_dir)).unwrap_or_default();
    log::info!("loaded config: {:?}", config);

    let selections = Selections::load(paths::selections(&cascade_dir)).unwrap_or_default();
    log::info!("loaded selections: {:?}", selections);

    let theme = Theme::load(paths::theme(&cascade_dir)).unwrap_or_default();
    log::info!("loaded theme: {:?}", theme);

    iced::application("cascade", Cascade::update, Cascade::view)
        .theme(Cascade::theme)
        .window(window::Settings {
            min_size: Some(Size::new(720., 520.)),
            icon: window::icon::from_file_data(CASCADE_ICON_BYTES, Some(image::ImageFormat::Ico))
                .ok(),
            ..Default::default()
        })
        .font(fonts::ICONS_FONT_BYTES)
        .scale_factor(Cascade::scale_factor)
        .subscription(Cascade::subscription)
        .run_with(move || Cascade::new((cascade_dir, config, selections, theme, debug)))?;

    Ok(())
}
