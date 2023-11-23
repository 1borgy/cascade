pub mod config;
pub mod dashboard;
use iced::Element;

use crate::config::CascadeConfig;

pub trait View {
    type Message;

    fn title(&self) -> String;

    fn set_config(&mut self, config: CascadeConfig);

    fn update(&mut self, message: Self::Message) -> Option<Self::Message>;

    fn view<'a>(&'a self) -> Element<'_, Self::Message>;
}
