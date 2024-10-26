//pub const IOSEVKA_REGULAR_BYTES: &[u8] =
//    include_bytes!("../../resources/IosevkaNerdFont-Regular.ttf");
//pub const IOSEVKA_BOLD_BYTES: &[u8] =
//    include_bytes!("../../resources/IosevkaNerdFont-Bold.ttf");

//pub static IOSEVKA_REGULAR: Font = Font {
//    family: Family::Name("Iosevka Nerd Font"),
//    weight: Weight::Normal,
//    stretch: Stretch::Normal,
//    style: Style::Normal,
//};
//pub static IOSEVKA_BOLD: Font = Font {
//    family: Family::Name("Iosevka Nerd Font"),
//    weight: Weight::Bold,
//    stretch: Stretch::Normal,
//    style: Style::Normal,
//};
use iced::{
    font::{Family, Stretch, Style, Weight},
    Font,
};

pub const ICONS_FONT_BYTES: &[u8] = include_bytes!("../../resources/icons.ttf");
pub const ICONS_FONT: Font = Font {
    family: Family::Name("icons"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
