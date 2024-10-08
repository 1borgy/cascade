use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum Theme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    #[default]
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
}

impl From<Theme> for iced::Theme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => iced::Theme::Light,
            Theme::Dark => iced::Theme::Dark,
            Theme::Dracula => iced::Theme::Dracula,
            Theme::Nord => iced::Theme::Nord,
            Theme::SolarizedLight => iced::Theme::SolarizedLight,
            Theme::SolarizedDark => iced::Theme::SolarizedDark,
            Theme::GruvboxLight => iced::Theme::GruvboxLight,
            Theme::GruvboxDark => iced::Theme::GruvboxDark,
            Theme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            Theme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            Theme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            Theme::TokyoNight => iced::Theme::TokyoNight,
            Theme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            Theme::TokyoNightLight => iced::Theme::TokyoNightLight,
            Theme::KanagawaWave => iced::Theme::KanagawaWave,
            Theme::KanagawaDragon => iced::Theme::KanagawaDragon,
            Theme::KanagawaLotus => iced::Theme::KanagawaLotus,
            Theme::Moonfly => iced::Theme::Moonfly,
            Theme::Nightfly => iced::Theme::Nightfly,
            Theme::Oxocarbon => iced::Theme::Oxocarbon,
        }
    }
}

impl Theme {
    pub const ALL: &'static [Self] = &[
        Self::Light,
        Self::Dark,
        Self::Dracula,
        Self::Nord,
        Self::SolarizedLight,
        Self::SolarizedDark,
        Self::GruvboxLight,
        Self::GruvboxDark,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::TokyoNightLight,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::KanagawaLotus,
        Self::Moonfly,
        Self::Nightfly,
        Self::Oxocarbon,
    ];
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
            Self::Dracula => write!(f, "Dracula"),
            Self::Nord => write!(f, "Nord"),
            Self::SolarizedLight => write!(f, "Solarized Light"),
            Self::SolarizedDark => write!(f, "Solarized Dark"),
            Self::GruvboxLight => write!(f, "Gruvbox Light"),
            Self::GruvboxDark => write!(f, "Gruvbox Dark"),
            Self::CatppuccinLatte => write!(f, "Catppuccin Latte"),
            Self::CatppuccinFrappe => write!(f, "Catppuccin Frappé"),
            Self::CatppuccinMacchiato => write!(f, "Catppuccin Macchiato"),
            Self::CatppuccinMocha => write!(f, "Catppuccin Mocha"),
            Self::TokyoNight => write!(f, "Tokyo Night"),
            Self::TokyoNightStorm => write!(f, "Tokyo Night Storm"),
            Self::TokyoNightLight => write!(f, "Tokyo Night Light"),
            Self::KanagawaWave => write!(f, "Kanagawa Wave"),
            Self::KanagawaDragon => write!(f, "Kanagawa Dragon"),
            Self::KanagawaLotus => write!(f, "Kanagawa Lotus"),
            Self::Moonfly => write!(f, "Moonfly"),
            Self::Nightfly => write!(f, "Nightfly"),
            Self::Oxocarbon => write!(f, "Oxocarbon"),
        }
    }
}

// use std::fmt::Display;
//
// use enum_iterator::Sequence;
// use hex_literal::hex;
// use serde::{Deserialize, Serialize};
//
// #[derive(Debug, Clone, Copy)]
// pub struct Rgb {
//     pub red: u8,
//     pub green: u8,
//     pub blue: u8,
// }
//
// impl Into<iced::Color> for Rgb {
//     fn into(self) -> iced::Color {
//         iced::Color::from_rgb8(self.red, self.green, self.blue)
//     }
// }
//
// pub struct Palette {
//     pub background: Rgb,
//     pub text: Rgb,
//     pub subtext: Rgb,
//     pub rosewater: Rgb,
//     pub flamingo: Rgb,
//     pub pink: Rgb,
//     pub mauve: Rgb,
//     pub red: Rgb,
//     pub maroon: Rgb,
//     pub peach: Rgb,
//     pub yellow: Rgb,
//     pub green: Rgb,
//     pub teal: Rgb,
//     pub sky: Rgb,
//     pub sapphire: Rgb,
//     pub blue: Rgb,
//     pub lavender: Rgb,
// }
//
// macro_rules! hexcolor {
//     ($code:literal) => {{
//         let [red, green, blue] = hex!($code);
//         Rgb { red, green, blue }
//     }};
// }
//
// impl Palette {
//     pub fn light() -> Self {
//         Self {
//             background: hexcolor!("eff1f5"),
//             text: hexcolor!("4c4f69"),
//             subtext: hexcolor!("6c6f85"),
//             rosewater: hexcolor!("dc8a78"),
//             flamingo: hexcolor!("dd7878"),
//             pink: hexcolor!("ea76cb"),
//             mauve: hexcolor!("8839ef"),
//             red: hexcolor!("d20f39"),
//             maroon: hexcolor!("e64553"),
//             peach: hexcolor!("fe640b"),
//             yellow: hexcolor!("df8e1d"),
//             green: hexcolor!("40a02b"),
//             teal: hexcolor!("179299"),
//             sky: hexcolor!("04a5e5"),
//             sapphire: hexcolor!("209fb5"),
//             blue: hexcolor!("1e66f5"),
//             lavender: hexcolor!("7287fd"),
//         }
//     }
//
//     pub fn dark() -> Self {
//         Self {
//             background: hexcolor!("24273a"),
//             text: hexcolor!("cad3f5"),
//             subtext: hexcolor!("a5adcb"),
//             rosewater: hexcolor!("f4dbd6"),
//             flamingo: hexcolor!("f0c6c6"),
//             pink: hexcolor!("f5bde6"),
//             mauve: hexcolor!("c6a0f6"),
//             red: hexcolor!("ed8796"),
//             maroon: hexcolor!("ee99a0"),
//             peach: hexcolor!("f5a97f"),
//             yellow: hexcolor!("eed49f"),
//             green: hexcolor!("a6da95"),
//             teal: hexcolor!("8bd5ca"),
//             sky: hexcolor!("91d7e3"),
//             sapphire: hexcolor!("7dc4e4"),
//             blue: hexcolor!("8aadf4"),
//             lavender: hexcolor!("b7bdf8"),
//         }
//     }
//
//     pub fn rgb(&self, color: ColorChoice) -> Rgb {
//         match color {
//             ColorChoice::Rosewater => self.rosewater,
//             ColorChoice::Flamingo => self.flamingo,
//             ColorChoice::Pink => self.pink,
//             ColorChoice::Mauve => self.mauve,
//             ColorChoice::Red => self.red,
//             ColorChoice::Maroon => self.maroon,
//             ColorChoice::Peach => self.peach,
//             ColorChoice::Yellow => self.yellow,
//             ColorChoice::Green => self.green,
//             ColorChoice::Teal => self.teal,
//             ColorChoice::Sky => self.sky,
//             ColorChoice::Sapphire => self.sapphire,
//             ColorChoice::Blue => self.blue,
//             ColorChoice::Lavender => self.lavender,
//         }
//     }
// }
//
// #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
// pub struct Theme {
//     pub background: BackgroundChoice,
//     pub color: ColorChoice,
// }
//
// impl From<&Theme> for iced::Theme {
//     fn from(theme: &Theme) -> Self {
//         let palette = theme.background.palette();
//         let primary = palette.rgb(theme.color);
//
//         iced::Theme::custom(
//             "cascade".to_string(),
//             iced::theme::Palette {
//                 background: palette.background.into(),
//                 text: palette.text.into(),
//                 primary: primary.into(),
//                 danger: palette.red.into(),
//                 success: palette.green.into(),
//             },
//         )
//     }
// }
//
// impl Theme {
//     pub fn primary(&self) -> iced::Color {
//         let palette = self.background.palette();
//         let color = palette.rgb(self.color);
//         color.into()
//     }
//
//     pub fn subtext(&self) -> iced::Color {
//         let palette = self.background.palette();
//         palette.subtext.into()
//     }
// }
//
// #[derive(
//     Debug, Clone, Copy, Default, PartialEq, Eq, Sequence, Serialize, Deserialize,
// )]
// pub enum BackgroundChoice {
//     Light,
//     Dark,
//     #[default]
//     System,
// }
//
// impl BackgroundChoice {
//     pub fn palette(&self) -> Palette {
//         match self {
//             BackgroundChoice::Light => Palette::light(),
//             BackgroundChoice::Dark => Palette::dark(),
//             BackgroundChoice::System => {
//                 // autodetect dark/light theme on system
//                 let mode = dark_light::detect();
//
//                 match mode {
//                     dark_light::Mode::Dark => {
//                         log::info!("autodetected system dark theme");
//                         Palette::dark()
//                     }
//                     dark_light::Mode::Light => {
//                         log::info!("autodetected system light theme");
//                         Palette::light()
//                     }
//                     dark_light::Mode::Default => {
//                         log::warn!("could not autodetect system theme; defaulting to light");
//                         Palette::light()
//                     }
//                 }
//             }
//         }
//     }
// }
//
// #[derive(
//     Debug, Clone, Copy, Default, PartialEq, Eq, Sequence, Serialize, Deserialize,
// )]
// pub enum ColorChoice {
//     Rosewater,
//     Flamingo,
//     Pink,
//     Mauve,
//     Red,
//     Maroon,
//     Peach,
//     Yellow,
//     Green,
//     Teal,
//     Sky,
//     Sapphire,
//     #[default]
//     Blue,
//     Lavender,
// }
//
// impl Display for BackgroundChoice {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 BackgroundChoice::Light => "light",
//                 BackgroundChoice::Dark => "dark",
//                 BackgroundChoice::System => "system",
//             }
//         )
//     }
// }
//
// impl Display for ColorChoice {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 ColorChoice::Rosewater => "rosewater",
//                 ColorChoice::Flamingo => "flamingo",
//                 ColorChoice::Pink => "pink",
//                 ColorChoice::Mauve => "mauve",
//                 ColorChoice::Red => "red",
//                 ColorChoice::Maroon => "maroon",
//                 ColorChoice::Peach => "peach",
//                 ColorChoice::Yellow => "yellow",
//                 ColorChoice::Green => "green",
//                 ColorChoice::Teal => "teal",
//                 ColorChoice::Sky => "sky",
//                 ColorChoice::Sapphire => "sapphire",
//                 ColorChoice::Blue => "blue",
//                 ColorChoice::Lavender => "lavender",
//             },
//         )
//     }
// }
