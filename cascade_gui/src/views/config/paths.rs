use std::{fmt::Display, path::PathBuf};

use cascade::config::{self, CascadePaths};
use enum_iterator::{all, Sequence};
use iced::{
    alignment,
    widget::{button, column, row, text},
    Element, Length, Renderer,
};
use iced_aw::Icon;
use rfd::FileDialog;

use crate::resources;

#[derive(Debug, Clone)]
enum FileDialogType {
    Directory,
    File(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum FileDialogTarget {
    Saves,
    Backup,
    Trickset,
}

impl Display for FileDialogTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FileDialogTarget::Saves => "thugpro saves",
                FileDialogTarget::Backup => "backup",
                FileDialogTarget::Trickset => "trickset",
            }
        )
    }
}

impl FileDialogTarget {
    // TODO: use AsRef<Path> instead of PathBuf?
    fn get_from_paths(&self, paths: &config::CascadePaths) -> Option<PathBuf> {
        match self {
            FileDialogTarget::Saves => paths.saves_dir.clone(),
            FileDialogTarget::Backup => paths.backup_dir.clone(),
            FileDialogTarget::Trickset => paths.trickset_path.clone(),
        }
    }

    fn set_in_paths(&self, paths: &mut config::CascadePaths, path: PathBuf) {
        match self {
            FileDialogTarget::Saves => paths.saves_dir = Some(path),
            FileDialogTarget::Backup => paths.backup_dir = Some(path),
            FileDialogTarget::Trickset => paths.trickset_path = Some(path),
        }
    }

    fn dialog_type(&self) -> FileDialogType {
        match self {
            FileDialogTarget::Saves | FileDialogTarget::Backup => {
                FileDialogType::Directory
            }
            FileDialogTarget::Trickset => {
                FileDialogType::File("SKA".to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathsMessage {
    OpenFileDialog(FileDialogTarget),

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

            PathsMessage::OpenFileDialog(target) => {
                let dialog = FileDialog::new();

                let selected_path = match target.dialog_type() {
                    FileDialogType::Directory => dialog.pick_folder(),
                    FileDialogType::File(extension) => dialog
                        .add_filter(extension.clone(), &[extension])
                        .pick_file(),
                };

                match selected_path {
                    // if the user didn't click cancel
                    Some(path) => {
                        // TODO: lol dont do it this way
                        target.set_in_paths(&mut self.paths, path);

                        Some(PathsMessage::PathsChanged(self.paths.clone()))
                    }
                    None => None,
                }
            }
        }
    }

    pub fn view(&self) -> Element<PathsMessage, Renderer> {
        column(
            all::<FileDialogTarget>()
                .map(move |target| {
                    column![
                        text(format!("{}", target)),
                        row![
                            button(resources::icon(Icon::Folder2Open))
                                .on_press(PathsMessage::OpenFileDialog(target))
                                .style(iced::theme::Button::Secondary),
                            button(text(
                                target
                                    .get_from_paths(&self.paths)
                                    .map(|path| format!("{:?}", path))
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
                })
                .collect(),
        )
        .spacing(10)
        .into()
    }
}
