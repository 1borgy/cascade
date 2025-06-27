use cascade_save as save;

pub trait Target {
    type Entry;
    type Error;

    fn name(&self, entry: &Self::Entry) -> impl AsRef<str>;
    fn list(&self) -> Result<impl IntoIterator<Item = Self::Entry>, Self::Error>;
    fn read(&self, entry: &Self::Entry) -> Result<save::Save, Self::Error>;
    fn write(&self, entry: &Self::Entry, save: &save::Save) -> Result<(), Self::Error>;
}

pub trait Structure {
    type Parsed;
    type Error;

    fn parse(&self, save: &save::Save) -> Result<Self::Parsed, Self::Error>;
    fn modify(&self, save: &mut save::Save, transform: Self::Parsed) -> Result<(), Self::Error>;
}
