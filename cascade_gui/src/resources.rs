use iced::{
    alignment,
    widget::{text, Text},
    Font,
};
use iced_aw::{graphics::icons::icon_to_char, Icon};

pub const WINDOW_ICON_BYTES: &[u8] =
    include_bytes!("../../resources/cascade.ico");
pub const ICONS_FONT: Font = Font::with_name("bootstrap-icons");

pub fn icon(_icon: Icon) -> Text<'static> {
    text(icon_to_char(_icon).to_string())
        .font(ICONS_FONT)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}
