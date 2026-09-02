use crate::{Result, save};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Flags {
    pub summary: bool,
    pub trickset: bool,
    pub scales: bool,
}

pub trait Cas: Sized + Send + Sync {
    type Save: save::Save;

    fn parse(save: &Self::Save) -> Result<Self>;
    fn modify(&self, save: &mut Self::Save) -> Result<()>;
    fn mask(self, flags: Flags) -> Self;
}
