use iced::{
    Font,
    font::{Family, Stretch, Style, Weight},
};

pub const ICONS_FONT_BYTES: &[u8] = include_bytes!("../../../assets/icons.ttf");
pub const ICONS_FONT: Font = Font {
    family: Family::Name("icons"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
