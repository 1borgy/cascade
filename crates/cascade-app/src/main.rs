#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{io, path::Path};

use app::Cascade;
use fern::colors::{Color, ColoredLevelConfig};
use iced::{Size, window};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

mod app;
mod config;
mod error;
mod fonts;
mod paths;
mod state;
mod tasks;
mod theme;
mod widget;

pub use config::Theme;
pub use error::{Error, Result};

use crate::{paths::Paths, state::State};

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

fn configure_logging(path: impl AsRef<Path>) -> color_eyre::Result<()> {
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

fn main() -> color_eyre::Result<()> {
    let cascade_dir = paths::cascade_dir().expect("could not determine cascade dir");
    let paths = Paths::new(&cascade_dir);

    configure_logging(&paths.log)?;
    log::info!("paths: {:?}", paths);

    let theme = Theme::load(&paths.theme).unwrap_or_default();
    log::info!("loaded theme: {:?}", theme);
    let state = State::load(&paths);
    log::info!("loaded state: {:?}", state);

    iced::application(
        move || Cascade::new(paths.clone(), theme.clone(), state.clone()),
        Cascade::update,
        Cascade::view,
    )
    .theme(Cascade::theme)
    .window(window::Settings {
        min_size: Some(Size::new(720., 520.)),
        icon: window::icon::from_file_data(CASCADE_ICON_BYTES, Some(image::ImageFormat::Ico)).ok(),
        ..Default::default()
    })
    .font(fonts::ICONS_FONT_BYTES)
    .scale_factor(Cascade::scale_factor)
    .subscription(Cascade::subscription)
    .run()?;

    Ok(())
}
