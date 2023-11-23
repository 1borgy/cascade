use crate::{mutations::spec::StructureMutation, save::SaveContent};

mod error;
mod spec;
mod trickset;

pub use error::MutationError;
pub use trickset::Trickset;

pub trait Mutation {
    fn data_mutation(&self) -> Option<&StructureMutation>;
    fn summary_mutation(&self) -> Option<&StructureMutation>;

    fn mutate(
        &self,
        content: SaveContent,
    ) -> Result<SaveContent, MutationError> {
        let data = match self.data_mutation() {
            Some(mutation) => Some(mutation.mutate_structure(content.data())?),
            None => None,
        };

        let summary = match self.summary_mutation() {
            Some(mutation) => {
                Some(mutation.mutate_structure(content.summary())?)
            }
            None => None,
        };

        let content =
            data.map(|data| content.with_data(data)).unwrap_or(content);

        let content = summary
            .map(|summary| content.with_summary(summary))
            .unwrap_or(content);

        Ok(content)
    }
}
