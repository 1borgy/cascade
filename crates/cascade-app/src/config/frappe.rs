use std::sync::LazyLock;

use iced::Color;

macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        LazyLock::new(|| Color::from_rgb8($r, $g, $b))
    };
}

pub static BASE: LazyLock<Color> = color!(48, 52, 70);
pub static SURFACE0: LazyLock<Color> = color!(65, 69, 89);
pub static TEXT: LazyLock<Color> = color!(198, 208, 245);
pub static BLUE: LazyLock<Color> = color!(140, 170, 238);
pub static GREEN: LazyLock<Color> = color!(166, 209, 137);
pub static YELLOW: LazyLock<Color> = color!(229, 200, 144);
pub static RED: LazyLock<Color> = color!(231, 130, 132);
pub static MAUVE: LazyLock<Color> = color!(202, 158, 230);
// pub static CRUST: LazyLock<Color> = color!(35, 38, 52);
// pub static MANTLE: LazyLock<Color> = color!(41, 44, 60);
// pub static SURFACE1: LazyLock<Color> = color!(81, 87, 109);
// pub static SURFACE2: LazyLock<Color> = color!(98, 104, 128);
// pub static OVERLAY0: LazyLock<Color> = color!(115, 121, 148);
// pub static OVERLAY1: LazyLock<Color> = color!(131, 139, 167);
// pub static OVERLAY2: LazyLock<Color> = color!(148, 156, 187);
// pub static SUBTEXT0: LazyLock<Color> = color!(165, 173, 206);
// pub static SUBTEXT1: LazyLock<Color> = color!(181, 191, 226);
// pub static LAVENDER: LazyLock<Color> = color!(186, 187, 241);
// pub static SAPPHIRE: LazyLock<Color> = color!(133, 193, 220);
// pub static SKY: LazyLock<Color> = color!(153, 209, 219);
// pub static TEAL: LazyLock<Color> = color!(129, 200, 190);
// pub static PEACH: LazyLock<Color> = color!(239, 159, 118);
// pub static MAROON: LazyLock<Color> = color!(234, 153, 156);
// pub static PINK: LazyLock<Color> = color!(244, 184, 228);
// pub static FLAMINGO: LazyLock<Color> = color!(238, 190, 190);
// pub static ROSEWATER: LazyLock<Color> = color!(242, 213, 207);
