use std::fmt;

// TODO: remove super uses
use super::SaveError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SaveFileExtension {
    SKA,
}

impl TryFrom<&str> for SaveFileExtension {
    type Error = SaveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "SKA" => Ok(SaveFileExtension::SKA),
            _ => Err(SaveError::UnknownFileExtension(value.to_string())),
        }
    }
}

impl fmt::Display for SaveFileExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SaveFileExtension::SKA => "SKA",
            }
        )
    }
}
