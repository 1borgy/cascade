use std::{fs, io, path::Path};

use iced::{application, Color};
use serde::{Deserialize, Serialize};

use crate::config::{frappe, Error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "color_serde", default = "default_background")]
    pub background: Color,
    #[serde(with = "color_serde", default = "default_text")]
    pub text: Color,
    #[serde(with = "color_serde", default = "default_primary")]
    pub primary: Color,
    #[serde(with = "color_serde", default = "default_secondary")]
    pub secondary: Color,
    #[serde(with = "color_serde", default = "default_success")]
    pub success: Color,
    #[serde(with = "color_serde", default = "default_danger")]
    pub danger: Color,
    #[serde(with = "color_serde", default = "default_warning")]
    pub warning: Color,
    #[serde(with = "color_serde", default = "default_highlight")]
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: default_background(),
            text: default_text(),
            primary: default_primary(),
            secondary: default_secondary(),
            success: default_success(),
            danger: default_danger(),
            warning: default_warning(),
            highlight: default_highlight(),
        }
    }
}

impl Theme {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;

        log::info!("reading theme from {:?}", path.as_ref());

        let contents = io::read_to_string(file)?;

        let config = toml::from_str(contents.as_str())?;

        Ok(config)
    }
}

fn default_background() -> iced::Color {
    *frappe::BASE
}

fn default_text() -> iced::Color {
    *frappe::TEXT
}

fn default_primary() -> iced::Color {
    *frappe::BLUE
}

fn default_secondary() -> iced::Color {
    *frappe::SURFACE0
}

fn default_success() -> iced::Color {
    *frappe::GREEN
}

fn default_danger() -> iced::Color {
    *frappe::RED
}

fn default_warning() -> iced::Color {
    *frappe::YELLOW
}

fn default_highlight() -> iced::Color {
    *frappe::MAUVE
}

impl application::DefaultStyle for Theme {
    fn default_style(&self) -> application::Appearance {
        application::Appearance {
            background_color: self.background,
            text_color: self.text,
        }
    }
}

pub fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() == 7 || hex.len() == 9 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);
        let a = (hex.len() == 9)
            .then(|| u8::from_str_radix(&hex[7..9], 16).ok())
            .flatten();

        return match (hash, r, g, b, a) {
            ("#", Ok(r), Ok(g), Ok(b), None) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }),
            ("#", Ok(r), Ok(g), Ok(b), Some(a)) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }),
            _ => None,
        };
    }

    None
}

pub fn color_to_hex(color: Color) -> String {
    use std::fmt::Write;

    let mut hex = String::with_capacity(9);

    let [r, g, b, a] = color.into_rgba8();

    let _ = write!(&mut hex, "#");
    let _ = write!(&mut hex, "{:02X}", r);
    let _ = write!(&mut hex, "{:02X}", g);
    let _ = write!(&mut hex, "{:02X}", b);

    if a < u8::MAX {
        let _ = write!(&mut hex, "{:02X}", a);
    }

    hex
}

mod color_serde {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)
            .map(|hex| super::hex_to_color(&hex))?
            .unwrap_or(Color::TRANSPARENT))
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::color_to_hex(*color).serialize(serializer)
    }
}
