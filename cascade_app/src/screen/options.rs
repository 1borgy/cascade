use std::{fmt::Debug, path::PathBuf};

use iced::{
    alignment::Vertical,
    widget::{self, button, pick_list, slider, text, Column, Container, Row},
    Alignment, Element, Length, Padding, Task,
};
use rfd::AsyncFileDialog;

use crate::{theme::Theme, widget::heading};

#[derive(Debug, Clone)]
pub enum Message {
    PickSavesDir,
    ClosedSavesDirDialog,

    SavesDirChanged(PathBuf),
    ThemeChanged(Theme),
    ScaleFactorChanged(f64),
}

#[derive(Debug, Clone)]
pub enum Event {
    SavesDirChanged(PathBuf),
    ThemeChanged(Theme),
    ScaleFactorChanged(f64),
}

pub struct Options {
    saves_dir: Option<PathBuf>,
    theme: Theme,
    scale_factor: f64,
}

impl Options {
    pub fn new(
        saves_dir: Option<PathBuf>,
        theme: Theme,
        scale_factor: f64,
    ) -> (Self, Task<Message>) {
        (
            Options {
                saves_dir,
                theme,
                scale_factor,
            },
            Task::none(),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::PickSavesDir => (
                Task::perform(pick_saves_dir(), |dir| {
                    dir.map(|dir| Message::SavesDirChanged(dir))
                        .unwrap_or(Message::ClosedSavesDirDialog)
                }),
                None,
            ),
            Message::ClosedSavesDirDialog => {
                log::debug!("user closed saves dir picker");
                (Task::none(), None)
            }
            Message::SavesDirChanged(saves_dir) => {
                self.saves_dir = Some(saves_dir.clone());
                (
                    Task::none(),
                    Some(Event::SavesDirChanged(saves_dir.clone())),
                )
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                (Task::none(), Some(Event::ThemeChanged(theme)))
            }
            Message::ScaleFactorChanged(scale_factor) => {
                self.scale_factor = scale_factor;
                (Task::none(), Some(Event::ScaleFactorChanged(scale_factor)))
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        Column::new()
            .push(section("saves directory", self.saves_dir_picker()))
            .push(section("theme", self.theme_picker()))
            .push(section("scale factor", self.scale_factor_slider()))
            .spacing(20)
            .into()
    }

    fn saves_dir_picker(&self) -> Element<Message> {
        // TODO: async
        file_picker(self.saves_dir.clone(), Message::PickSavesDir)
    }

    fn theme_picker(&self) -> Element<Message> {
        pick_list(Theme::ALL, Some(self.theme), Message::ThemeChanged).into()
    }

    fn scale_factor_slider(&self) -> Element<Message> {
        Row::new()
            .push(
                slider(
                    0.5..=5.,
                    self.scale_factor,
                    Message::ScaleFactorChanged,
                )
                .step(0.1),
            )
            .push(text(format!("{:.1}", self.scale_factor)))
            .spacing(5)
            .into()
    }
}

fn section<'a>(
    label: &'a str,
    element: Element<'a, Message>,
) -> Element<'a, Message> {
    Column::new()
        .push(heading(label))
        .push(element)
        .spacing(10)
        .into()
}

fn file_picker<'a>(
    current_path: Option<PathBuf>,
    on_press: Message,
) -> Element<'a, Message> {
    let icon_button = button(text(" ").size(24))
        .padding(Padding {
            left: 7.,
            right: 5.,
            top: 3.,
            bottom: 1.,
        })
        .on_press(on_press)
        .style(widget::button::secondary);

    let path_text = text(
        current_path
            .map(move |path| {
                path.into_os_string().into_string().unwrap_or("??".into())
            })
            .unwrap_or("not set...".into()),
    );

    let path_text = Container::new(path_text)
        .style(widget::container::bordered_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            left: 10.,
            right: 10.,
            top: 0.,
            bottom: 0.,
        })
        .align_y(Vertical::Center);

    Row::new()
        .push(icon_button)
        .push(path_text)
        .spacing(10)
        .align_y(Alignment::Center)
        .height(Length::Shrink)
        .into()
}

async fn pick_saves_dir() -> Option<PathBuf> {
    Some(AsyncFileDialog::new().pick_folder().await?.path().into())
}
