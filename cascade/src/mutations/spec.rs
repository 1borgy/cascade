use std::{collections::HashMap, sync::Arc};

use super::error::MutationError;
use crate::structure::{
    NameChecksum, Structure, StructureError, Symbol, Value,
};

// We define two types of datatypes for structure mutation here: `spec` and `mutation`.
//
// A spec simply defines the format of which keys should be mutated, and which actions
// should be performed on those keys. It does not specify what values to copy.
//
// A mutation contains the same information, but also contains information about the
// values to set.
//
// A spec can be populated from a structure to create a mutation. The spec actions will
// be converted to mutation actions (or errors) using the values in the structure. For
// example, a `CopyOrDelete` spec action will either result in a `Set` or `Delete`
// mutation action depending on whether the given key is present. A `CopyRequired` will
// either result in a `Set` or `MutationError` depending on whether the given key is
// present.

#[derive(Copy, Clone, Debug)]
pub enum SpecAction {
    #[allow(dead_code)]
    // TODO: i promise this will be used in the cas randomizer i swear!!!!
    CopyOrRemove,
    CopyRequired,
}

impl SpecAction {
    pub fn populate(
        &self,
        name_checksum: NameChecksum,
        symbol: Option<&Symbol>,
    ) -> Result<MutationAction, MutationError> {
        match self {
            SpecAction::CopyOrRemove => match symbol {
                Some(symbol) => Ok(MutationAction::Set(symbol.clone())),
                None => Ok(MutationAction::Remove),
            },
            SpecAction::CopyRequired => match symbol {
                Some(symbol) => Ok(MutationAction::Set(symbol.clone())),
                None => Err(MutationError::SymbolNotFound(name_checksum)),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpecChildren {
    children: HashMap<NameChecksum, SpecNode>,
}

impl SpecChildren {
    fn build<I>(children: I) -> Result<SpecChildren, MutationError>
    where
        I: IntoIterator<Item = SpecNode>,
    {
        Ok(SpecChildren {
            children: children
                .into_iter()
                .map(|child| (child.name(), child))
                .collect(),
        })
    }

    pub fn populate_from_structure(
        &self,
        structure: Arc<Structure>,
    ) -> Result<MutationChildren, MutationError> {
        Ok(MutationChildren {
            children: self
                .children
                .iter()
                .map(|(name, node)| try {
                    (*name, node.populate_from_symbol(structure.get(*name))?)
                })
                .collect::<Result<_, MutationError>>()?,
        })
    }
}

#[derive(Clone, Debug)]
pub enum SpecNode {
    Node {
        name_checksum: NameChecksum,
        children: SpecChildren,
    },
    Leaf {
        name_checksum: NameChecksum,
        action: SpecAction,
    },
}

// Root
pub struct Spec {
    children: SpecChildren,
}

impl Spec {
    pub fn build<I>(children: I) -> Result<Self, MutationError>
    where
        I: IntoIterator<Item = SpecNode>,
    {
        Ok(Self {
            children: SpecChildren::build(children)?,
        })
    }

    pub fn populate_from_structure(
        &self,
        structure: Arc<Structure>,
    ) -> Result<StructureMutation, MutationError> {
        Ok(StructureMutation {
            children: self.children.populate_from_structure(structure)?,
        })
    }
}

impl SpecNode {
    pub fn node<N, I>(name: N, children: I) -> Result<Self, MutationError>
    where
        I: IntoIterator<Item = SpecNode>,
        N: TryInto<NameChecksum, Error = StructureError>,
    {
        Ok(SpecNode::Node {
            name_checksum: name.try_into()?,
            children: SpecChildren::build(children)?,
        })
    }

    pub fn leaf<N>(name: N, action: SpecAction) -> Result<Self, MutationError>
    where
        N: TryInto<NameChecksum, Error = StructureError>,
    {
        Ok(SpecNode::Leaf {
            name_checksum: name.try_into()?,
            action,
        })
    }

    pub fn name(&self) -> NameChecksum {
        match self {
            SpecNode::Node {
                name_checksum,
                children: _,
            } => *name_checksum,
            SpecNode::Leaf {
                name_checksum,
                action: _,
            } => *name_checksum,
        }
    }

    pub fn populate_from_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Result<MutationNode, MutationError> {
        match self {
            SpecNode::Node {
                name_checksum,
                children,
            } => SpecNode::populate_node(symbol, *name_checksum, children),
            SpecNode::Leaf {
                name_checksum,
                action,
            } => SpecNode::populate_leaf(symbol, *name_checksum, action),
        }
    }

    fn populate_node(
        symbol: Option<&Symbol>,
        name_checksum: NameChecksum,
        children: &SpecChildren,
    ) -> Result<MutationNode, MutationError> {
        match symbol {
            Some(symbol) => Ok(MutationNode::Node {
                name_checksum,
                children: children
                    .populate_from_structure(symbol.try_as_struct()?)?,
            }),
            None => Err(MutationError::SymbolNotFound(name_checksum)),
        }
    }

    fn populate_leaf(
        symbol: Option<&Symbol>,
        name_checksum: NameChecksum,
        action: &SpecAction,
    ) -> Result<MutationNode, MutationError> {
        Ok(MutationNode::Leaf {
            name_checksum,
            action: action.populate(name_checksum, symbol)?,
        })
    }
}

#[derive(Clone, Debug)]
pub enum MutationAction {
    Set(Symbol),
    Remove,
}

#[derive(Clone, Debug)]
pub struct MutationChildren {
    children: HashMap<NameChecksum, MutationNode>,
}

impl MutationChildren {
    pub fn mutate_structure(
        &self,
        structure: Arc<Structure>,
    ) -> Result<Arc<Structure>, MutationError> {
        let mut children = self.children.clone();

        let existing_children = structure
            .iter()
            .map(|symbol| {
                let name_checksum = symbol.name_checksum();

                match children.remove(&name_checksum) {
                    Some(node) => {
                        Ok(node.mutate_symbol(structure.get(name_checksum))?)
                    }
                    None => Ok(Some(symbol.clone())),
                }
            })
            .collect::<Result<Vec<Option<Symbol>>, MutationError>>()?;

        let missing_children = children
            .drain()
            .map(|(_, node)| node.mutate_symbol(None))
            .collect::<Result<Vec<Option<Symbol>>, MutationError>>()?;

        Ok(existing_children
            .into_iter()
            .chain(missing_children)
            .filter_map(|symbol| symbol)
            .into_iter()
            .collect())
    }
}

#[derive(Clone, Debug)]
pub enum MutationNode {
    Node {
        name_checksum: NameChecksum,
        children: MutationChildren,
    },
    Leaf {
        name_checksum: NameChecksum,
        action: MutationAction,
    },
}

impl MutationNode {
    pub fn mutate_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Result<Option<Symbol>, MutationError> {
        match self {
            MutationNode::Node {
                name_checksum: name,
                children,
            } => Ok(Some(MutationNode::mutate_node(symbol, *name, children)?)),
            MutationNode::Leaf {
                name_checksum: _,
                action,
            } => Ok(MutationNode::mutate_leaf(action)),
        }
    }

    pub fn mutate_node(
        symbol: Option<&Symbol>,
        name_checksum: NameChecksum,
        children: &MutationChildren,
    ) -> Result<Symbol, MutationError> {
        match symbol {
            Some(symbol) => Ok(symbol.with_value(Value::Structure(
                children.mutate_structure(symbol.try_as_struct()?)?,
            ))),
            // TODO: Should we add a structure if it does not exist?
            None => Err(MutationError::SymbolNotFound(name_checksum)),
        }
    }

    pub fn mutate_leaf(action: &MutationAction) -> Option<Symbol> {
        match action {
            MutationAction::Set(inner) => Some(inner.clone()),
            MutationAction::Remove => None,
        }
    }
}

// Root
pub struct StructureMutation {
    children: MutationChildren,
}

impl StructureMutation {
    pub fn mutate_structure(
        &self,
        structure: Arc<Structure>,
    ) -> Result<Arc<Structure>, MutationError> {
        self.children.mutate_structure(structure)
    }
}
