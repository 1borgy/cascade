use iced::{
    alignment,
    widget::{button, column, row, text},
    Element, Length, Renderer,
};
use iced_aw::Icon;
use rfd::FileDialog;

use crate::{config::CascadePaths, paths, resources};

#[derive(Debug, Clone)]
pub enum PathsMessage {
    SetSavesDir,

    OpenCascadeDir,
    OpenSavesDir,

    PathsChanged(CascadePaths),
}

pub struct PathsComponent {
    paths: CascadePaths,
}

impl PathsComponent {
    pub fn new(paths: CascadePaths) -> Self {
        Self { paths }
    }

    pub fn set_paths(&mut self, paths: CascadePaths) {
        self.paths = paths;
    }

    pub fn update(&mut self, message: PathsMessage) -> Option<PathsMessage> {
        match message {
            // this is a message we only send to the parent component
            PathsMessage::PathsChanged(_) => None,

            PathsMessage::SetSavesDir => {
                match FileDialog::new().pick_folder() {
                    // if the user didn't click cancel
                    Some(path) => {
                        self.paths.set_saves_dir(path);

                        Some(PathsMessage::PathsChanged(self.paths.clone()))
                    }
                    None => None,
                }
            }
            PathsMessage::OpenCascadeDir => {
                match paths::detect_cascade_dir() {
                    Ok(dir) => {
                        if let Err(err) = open::that(dir) {
                            log::error!("could not open cascade dir: {}", err)
                        }
                    }
                    Err(err) => {
                        log::error!("could not open cascade dir: {}", err)
                    }
                };

                None
            }
            PathsMessage::OpenSavesDir => {
                match self.paths.saves_dir() {
                    Some(dir) => {
                        if let Err(err) = open::that(dir) {
                            log::error!("could not open cascade dir: {}", err)
                        }
                    }
                    None => log::error!(
                        "could not open saves dir; no saves dir to open"
                    ),
                };

                None
            }
        }
    }

    pub fn view(&self) -> Element<PathsMessage, Renderer> {
        column![view_saves_picker(&self.paths), view_buttons()]
            .spacing(30)
            .into()
    }
}

pub fn view_saves_picker(
    paths: &CascadePaths,
) -> Element<PathsMessage, Renderer> {
    column![
        text(format!("thugpro saves")),
        row![
            button(resources::icon(Icon::Folder2Open))
                .on_press(PathsMessage::SetSavesDir)
                .style(iced::theme::Button::Secondary),
            button(text(
                paths
                    .saves_dir()
                    .map(|path| path
                        .into_os_string()
                        .into_string()
                        .unwrap_or("<unknown path>".to_string()))
                    .unwrap_or("not set...".to_string())
            ))
            .style(iced::theme::Button::Secondary)
            .width(Length::Fill)
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
    ]
    .spacing(5)
    .into()
}

pub fn view_buttons<'a>() -> Element<'a, PathsMessage, Renderer> {
    row![
        button("open cascade folder")
            .on_press(PathsMessage::OpenCascadeDir)
            .padding(10),
        button("open saves folder")
            .on_press(PathsMessage::OpenSavesDir)
            .padding(10),
    ]
    .spacing(20)
    .into()
}
