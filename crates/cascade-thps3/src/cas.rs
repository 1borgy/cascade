use cascade_qb as qb;

use crate::{Save, id};

fn expect_symbol(parent: &qb::Structure, id: qb::Id) -> cascade_core::Result<&qb::Symbol> {
    parent
        .get(id)
        .ok_or(cascade_core::Error::SymbolNotFound(id))
}

fn expect_symbol_mut(
    parent: &mut qb::Structure,
    id: qb::Id,
) -> cascade_core::Result<&mut qb::Symbol> {
    parent
        .get_mut(id)
        .ok_or(cascade_core::Error::SymbolNotFound(id))
}

// expect symbol and expect structure
fn expect_structure(parent: &qb::Structure, id: qb::Id) -> cascade_core::Result<&qb::Structure> {
    let symbol = expect_symbol(parent, id)?;
    Ok(symbol.value.try_as_structure()?)
}

fn expect_structure_mut(
    parent: &mut qb::Structure,
    id: qb::Id,
) -> cascade_core::Result<&mut qb::Structure> {
    let symbol = expect_symbol_mut(parent, id)?;
    Ok(symbol.value.try_as_structure_mut()?)
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Item {
    /// Require a symbol to be present.
    Present(qb::Symbol),
    /// Require a symbol to be vacant (remove if present).
    Vacant,
    /// No modification.
    #[default]
    Ignore,
}

impl Item {
    pub fn modify(&self, structure: &mut qb::Structure, id: qb::Id) {
        match self {
            Item::Present(symbol) => {
                structure.insert(symbol.clone());
            }
            Item::Vacant => {
                structure.remove(id);
            }
            Item::Ignore => (),
        }
    }
}

impl From<qb::Symbol> for Item {
    fn from(value: qb::Symbol) -> Self {
        Self::Present(value)
    }
}

impl From<Option<qb::Symbol>> for Item {
    fn from(value: Option<qb::Symbol>) -> Self {
        match value {
            Some(symbol) => Self::from(symbol),
            None => Self::Vacant,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Cas {
    pub custom: Custom,
}

impl cascade_core::Cas for Cas {
    type Save = Save;

    fn parse(save: &Self::Save) -> cascade_core::Result<Self> {
        Self::try_from(save)
    }

    fn modify(&self, save: &mut Self::Save) -> cascade_core::Result<()> {
        self.custom
            .modify(expect_structure_mut(&mut save.structure, id::CUSTOM)?)?;

        Ok(())
    }

    fn mask(self, flags: cascade_core::Flags) -> Self {
        Cas {
            custom: Custom {
                appearance: Appearance {
                    height_scale: if flags.scales {
                        self.custom.appearance.height_scale
                    } else {
                        Default::default()
                    },
                    weight_scale: if flags.scales {
                        self.custom.appearance.weight_scale
                    } else {
                        Default::default()
                    },
                    scaling_mode: if flags.scales {
                        self.custom.appearance.scaling_mode
                    } else {
                        Default::default()
                    },
                },
                info: Info {
                    trick_mapping: if flags.trickset {
                        self.custom.info.trick_mapping
                    } else {
                        Default::default()
                    },
                    max_specials: if flags.trickset {
                        self.custom.info.max_specials
                    } else {
                        Default::default()
                    },
                    specials: if flags.trickset {
                        self.custom.info.specials
                    } else {
                        Default::default()
                    },
                },
            },
        }
    }
}

impl TryFrom<&Save> for Cas {
    type Error = cascade_core::Error;

    fn try_from(save: &Save) -> cascade_core::Result<Self> {
        Ok(Self {
            custom: Custom::try_from(expect_structure(&save.structure, id::CUSTOM)?)?,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Custom {
    pub appearance: Appearance,
    pub info: Info,
}

impl TryFrom<&qb::Structure> for Custom {
    type Error = cascade_core::Error;

    fn try_from(custom: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            appearance: Appearance::try_from(expect_structure(custom, id::APPEARANCE)?)?,
            info: Info::try_from(expect_structure(custom, id::INFO)?)?,
        })
    }
}

impl Custom {
    pub fn modify(&self, custom: &mut qb::Structure) -> cascade_core::Result<()> {
        self.appearance
            .modify(expect_structure_mut(custom, id::APPEARANCE)?);
        self.info.modify(expect_structure_mut(custom, id::INFO)?);

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    pub trick_mapping: Item,
    pub max_specials: Item,
    pub specials: Item,
}

impl TryFrom<&qb::Structure> for Info {
    type Error = cascade_core::Error;

    fn try_from(info: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            trick_mapping: info.get(id::TRICK_MAPPING).cloned().into(),
            max_specials: info.get(id::MAX_SPECIALS).cloned().into(),
            specials: info.get(id::SPECIALS).cloned().into(),
        })
    }
}

impl Info {
    pub fn modify(&self, info: &mut qb::Structure) {
        self.trick_mapping.modify(info, id::TRICK_MAPPING);
        self.specials.modify(info, id::SPECIALS);
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Appearance {
    pub height_scale: Item,
    pub weight_scale: Item,
    pub scaling_mode: Item,
}

impl Appearance {
    pub fn modify(&self, appearance: &mut qb::Structure) {
        self.height_scale.modify(appearance, id::HEIGHT_SCALE);
        self.weight_scale.modify(appearance, id::WEIGHT_SCALE);
        self.scaling_mode.modify(appearance, id::SCALING_MODE);
    }
}

impl TryFrom<&qb::Structure> for Appearance {
    type Error = cascade_core::Error;

    fn try_from(structure: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            height_scale: structure.get(id::HEIGHT_SCALE).cloned().into(),
            weight_scale: structure.get(id::WEIGHT_SCALE).cloned().into(),
            scaling_mode: structure.get(id::SCALING_MODE).cloned().into(),
        })
    }
}
