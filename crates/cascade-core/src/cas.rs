use crate::save;

pub struct Flags {
    pub summary: bool,
    pub trickset: bool,
    pub scales: bool,
}

pub trait Cas: Sized {
    type Error;
    type Save: save::Save;

    fn parse(save: &Self::Save) -> Result<Self, Self::Error>;
    fn modify(&self, save: &mut Self::Save) -> Result<(), Self::Error>;
    fn mask(self, flags: Flags) -> Self;
}
