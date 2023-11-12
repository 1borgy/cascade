use std::{backtrace::Backtrace, fs, io};

use cascade::{
    config::CascadeConfig,
    files::{
        load_save, load_saves_from_dir, with_copied_tricksets,
        write_saves_to_dir,
    },
};
use iced::{
    alignment::{self, Alignment},
    widget::{button, column, row, text},
    Element, Length, Renderer,
};
use rfd::FileDialog;
use thiserror::Error;

use crate::{
    about,
    theming::{config_to_primary_color, config_to_subtext_color},
    views::View,
};

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("saves path is not set!")]
    SavesDirNotSet,
    #[error("trick source path is not set!")]
    TricksetNotSet,
    #[error("backups path is not set!")]
    BackupsDirNotSet,
    #[error("an io error occurred: {source}")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    SetTrickSource,
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

    // TODO: put like All of this somewhere else...
    fn set_trickset(&mut self) -> Result<(), DashboardError> {
        let trickset_path = self
            .config
            .paths
            .trick_source
            .clone()
            .ok_or(DashboardError::TricksetNotSet)?;

        let dialog = FileDialog::new();

        if let Some(selected_path) =
            dialog.add_filter("SKA", &["SKA"]).pick_file()
        {
            fs::copy(selected_path, trickset_path)?;
            self.status_text = "successfully set trickset".to_string();
        }

        Ok(())
    }

    fn backup(&self) -> Result<(), DashboardError> {
        let backup_dir = self
            .config
            .paths
            .backup_dir
            .clone()
            .ok_or(DashboardError::BackupsDirNotSet)?;

        let saves_dir = self
            .config
            .paths
            .saves_dir
            .clone()
            .ok_or(DashboardError::SavesDirNotSet)?;

        let datetime = time::OffsetDateTime::now_local().unwrap_or({
            log::info!("could not get local timezone; using utc");
            time::OffsetDateTime::now_utc()
        });

        let subdir_name = format!(
            "{:04}-{:02}-{:02}-{:02}-{:02}-{:02}",
            datetime.year(),
            u8::from(datetime.month()),
            datetime.day(),
            datetime.hour(),
            datetime.minute(),
            datetime.second()
        );

        let mut subdir = backup_dir.clone();
        subdir.push(subdir_name);

        fs::create_dir_all(&subdir)?;

        for file in fs::read_dir(saves_dir)? {
            // why him so confused ??
            let file = file?;
            let file_path = file.path();

            if file.file_type()?.is_file() {
                if let Some(extension) = file.path().extension() {
                    if extension == "SKA" {
                        // holy shit let it stop im so sorry
                        // just use .filter() or something u moron
                        if let Some(file_name) = file_path.file_name() {
                            let mut backup_file_path = subdir.clone();
                            backup_file_path.push(file_name);

                            log::info!(
                                "backing up {:?} to {:?}",
                                file_path,
                                backup_file_path
                            );

                            fs::copy(file_path, backup_file_path)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn copy_trickset(&self) -> Result<(), DashboardError> {
        self.backup()?;

        // TODO: probably could be a method or something
        let saves_dir = self
            .config
            .paths
            .saves_dir
            .clone()
            .ok_or(DashboardError::SavesDirNotSet)?;

        let trick_source = self
            .config
            .paths
            .trick_source
            .clone()
            .ok_or(DashboardError::TricksetNotSet)?;

        let saves = load_saves_from_dir(&saves_dir).map_err(|err| {
            DashboardError::Other(format!("error loading saves: {}", err))
        })?;

        let trickset = load_save(&trick_source).map_err(|err| {
            DashboardError::Other(format!(
                "error loading trick source: {}",
                err
            ))
        })?;

        // TODO: why does this Not Return A Result or something
        let copied_saves = with_copied_tricksets(&saves, &trickset);

        write_saves_to_dir(&copied_saves, &saves_dir).map_err(|err| {
            DashboardError::Other(format!(
                "error writing saves to dir: {}",
                err
            ))
        })?;

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
            DashboardMessage::SetTrickSource => match self.set_trickset() {
                Err(err) => self.status_text = err.to_string(),
                Ok(_) => (),
            },
            DashboardMessage::CopyTrickset => match self.copy_trickset() {
                Err(err) => self.status_text = err.to_string(),
                Ok(_) => {
                    self.status_text =
                        "successfully copied trickset to all saves".to_string()
                }
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
                    .on_press(DashboardMessage::SetTrickSource),
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
