#![feature(error_generic_member_access)]

use gui::Cascade;

mod about;
mod gui;
mod resources;
mod theming;
mod views;

// TODO: turn this into CascadeResult or something
pub fn run() -> iced::Result {
    Cascade::start()
}
