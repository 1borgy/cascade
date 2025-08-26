use crate::{Flags, save};

pub trait Cas<Save, Error>
where
    Save: save::Save<Error>,
{
    fn parse(save: &Save) -> Result<impl Cas<Save, Error>, Error>;
    fn modify(&self, save: &mut Save) -> Result<(), Error>;
    fn mask(self, flags: Flags) -> Self;
}
