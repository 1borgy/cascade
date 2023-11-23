use std::sync::Arc;

use super::{spec::StructureMutation, Mutation};
use crate::{
    mutations::{
        error::MutationError,
        spec::{Spec, SpecAction, SpecNode},
    },
    save::SaveContent,
};

pub struct Trickset {
    data: StructureMutation,
}

impl Trickset {
    pub fn from_save(save: &SaveContent) -> Result<Self, MutationError> {
        let spec = Spec::build(vec![
            SpecNode::node(
                "CustomSkater",
                vec![
                    (SpecNode::node(
                        "custom",
                        vec![
                            (SpecNode::node(
                                "info",
                                vec![
                                    SpecNode::leaf(
                                        "trick_mapping",
                                        SpecAction::CopyRequired,
                                    )?,
                                    SpecNode::leaf(
                                        "specials",
                                        SpecAction::CopyRequired,
                                    )?,
                                ],
                            )?),
                        ],
                    )?),
                ],
            )?,
            SpecNode::node(
                "StorySkater",
                vec![SpecNode::leaf("tricks", SpecAction::CopyRequired)?],
            )?,
        ])?;

        Ok(Self {
            data: spec.populate_from_structure(Arc::clone(&save.data))?,
        })
    }
}

impl Mutation for Trickset {
    fn data_mutation(&self) -> Option<&StructureMutation> {
        Some(&self.data)
    }

    fn summary_mutation(&self) -> Option<&StructureMutation> {
        None
    }
}
