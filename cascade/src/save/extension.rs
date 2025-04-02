use std::fmt;

use super::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Extension {
    SKA,
}

impl TryFrom<&str> for Extension {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "SKA" => Ok(Extension::SKA),
            _ => Err(Error::UnknownFileExtension(value.to_string())),
        }
    }
}

impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Extension::SKA => "SKA",
            }
        )
    }
}
