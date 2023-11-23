use std::{backtrace::Backtrace, io, path::PathBuf};

use cascade::actions::{self, ActionError};
use iced::{
    alignment::{self, Alignment},
    widget::{button, column, row, text},
    Element, Length, Renderer,
};
use rfd::FileDialog;
use thiserror::Error;

use crate::{
    about,
    config::CascadeConfig,
    theming::{config_to_primary_color, config_to_subtext_color},
    views::View,
};

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("saves path is not set!")]
    SavesDirNotSet,
    #[error("trickset path is not set!")]
    TricksetNotSet,
    #[error("backups path is not set!")]
    BackupsDirNotSet,
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
    #[error("an action error occurred: {source}")]
    Action {
        #[from]
        source: ActionError,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    SetTrickset,
    CopyTrickset,
}

#[derive(Debug, Clone)]
pub struct DashboardView {
    config: CascadeConfig,

    status_text: String,

    primary_color: iced::Color,
    subtext_color: iced::Color,
}

impl DashboardView {
    pub fn new(config: CascadeConfig) -> Self {
        // TODO: code reuse
        let primary_color = config_to_primary_color(&config.theme);
        let subtext_color = config_to_subtext_color(&config.theme);

        let status_text = "".to_string();

        Self {
            config,
            primary_color,
            subtext_color,
            status_text,
        }
    }

    fn backup_dir(&self) -> Result<PathBuf, DashboardError> {
        self.config
            .paths
            .backup_dir
            .clone()
            .ok_or(DashboardError::BackupsDirNotSet)
    }

    fn saves_dir(&self) -> Result<PathBuf, DashboardError> {
        self.config
            .paths
            .saves_dir
            .clone()
            .ok_or(DashboardError::SavesDirNotSet)
    }

    fn trickset_path(&self) -> Result<PathBuf, DashboardError> {
        self.config
            .paths
            .trickset_path
            .clone()
            .ok_or(DashboardError::TricksetNotSet)
    }

    fn set_trickset(&mut self) -> Result<(), DashboardError> {
        let trickset_path = self.trickset_path()?;

        let dialog =
            FileDialog::new().add_filter("SKA", &["SKA"]).set_directory(
                self.config
                    .paths
                    .saves_dir
                    .clone()
                    .ok_or(DashboardError::SavesDirNotSet)?,
            );

        if let Some(selected_path) = dialog.pick_file() {
            actions::set_trickset(trickset_path, selected_path)?;
            self.status_text = "successfully set trickset".to_string();
        }

        Ok(())
    }

    fn copy_trickset(&mut self) -> Result<(), DashboardError> {
        let saves_dir = self.saves_dir()?;
        let backup_dir = self.backup_dir()?;
        let trickset_path = self.trickset_path()?;

        let (num_successful_saves, num_all_saves) =
            actions::copy_trickset(trickset_path, backup_dir, saves_dir)?;

        log::info!("return back to dashboard");

        self.status_text = format!(
            "successfully copied trickset to {}/{} saves",
            num_successful_saves, num_all_saves
        );

        Ok(())
    }
}

impl View for DashboardView {
    type Message = DashboardMessage;

    fn title(&self) -> String {
        "dashboard".to_string()
    }

    fn set_config(&mut self, config: CascadeConfig) {
        self.primary_color = config_to_primary_color(&config.theme);
        self.subtext_color = config_to_subtext_color(&config.theme);

        self.config = config
    }

    fn update(&mut self, message: Self::Message) -> Option<DashboardMessage> {
        match message {
            DashboardMessage::SetTrickset => match self.set_trickset() {
                Err(err) => self.status_text = err.to_string(),
                Ok(_) => (),
            },
            DashboardMessage::CopyTrickset => match self.copy_trickset() {
                Err(err) => self.status_text = err.to_string(),
                Ok(_) => {}
            },
        }
        None
    }

    fn view(&self) -> Element<DashboardMessage, Renderer> {
        row![column![
            //
            column![
                row![
                    text("cascade")
                        .size(100)
                        .style(iced::theme::Text::Color(self.primary_color)),
                    text(format!("v{}", about::VERSION))
                        .size(30)
                        .style(iced::theme::Text::Color(self.subtext_color))
                ]
                .align_items(Alignment::Center)
                .spacing(10),
                //
                row![
                    iced::widget::horizontal_space(5),
                    text("by borgy")
                        .style(iced::theme::Text::Color(self.subtext_color)),
                ]
            ],
            //
            row![
                button(text("set trickset").size(22))
                    .padding([10, 10])
                    .on_press(DashboardMessage::SetTrickset),
                button(text("copy trickset to saves").size(22))
                    .padding([10, 10])
                    .on_press(DashboardMessage::CopyTrickset)
            ]
            .spacing(10),
            text(self.status_text.clone())
                .style(iced::theme::Text::Color(self.subtext_color))
                .horizontal_alignment(alignment::Horizontal::Center),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .spacing(30),]
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center)
        .into()
    }
}
