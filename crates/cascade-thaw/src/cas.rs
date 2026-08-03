use cascade_qb as qb;

use crate::{chunk, id, save::Save};

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
        if let Some(symbol) = value {
            Self::from(symbol)
        } else {
            Self::Vacant
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Cas {
    pub summary: Summary,
    pub data: Data,
}

impl cascade_core::Cas for Cas {
    type Save = Save;

    fn parse(save: &Self::Save) -> cascade_core::Result<Self> {
        Self::try_from(save)
    }

    fn modify(&self, save: &mut Self::Save) -> cascade_core::Result<()> {
        self.modify(save)
    }

    fn mask(self, flags: cascade_core::Flags) -> Self {
        Cas {
            summary: if flags.summary {
                self.summary
            } else {
                Default::default()
            },
            data: Data {
                custom_skater: CustomSkater {
                    custom_classic: CustomClassic {
                        info: Info {
                            trick_mapping: if flags.trickset {
                                self.data.custom_skater.custom_classic.info.trick_mapping
                            } else {
                                Default::default()
                            },
                            specials: if flags.trickset {
                                self.data.custom_skater.custom_classic.info.specials
                            } else {
                                Default::default()
                            },
                        },
                        appearance: Appearance {
                            board_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .board_bone_group
                            } else {
                                Default::default()
                            },
                            feet_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .feet_bone_group
                            } else {
                                Default::default()
                            },
                            hands_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .hands_bone_group
                            } else {
                                Default::default()
                            },
                            head_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .head_bone_group
                            } else {
                                Default::default()
                            },
                            headtop_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .headtop_bone_group
                            } else {
                                Default::default()
                            },
                            jaw_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .jaw_bone_group
                            } else {
                                Default::default()
                            },
                            lower_arm_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .lower_arm_bone_group
                            } else {
                                Default::default()
                            },
                            lower_leg_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .lower_leg_bone_group
                            } else {
                                Default::default()
                            },
                            nose_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .nose_bone_group
                            } else {
                                Default::default()
                            },
                            object_scaling: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .object_scaling
                            } else {
                                Default::default()
                            },
                            stomach_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .stomach_bone_group
                            } else {
                                Default::default()
                            },
                            torso_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .torso_bone_group
                            } else {
                                Default::default()
                            },
                            upper_arm_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .upper_arm_bone_group
                            } else {
                                Default::default()
                            },
                            upper_leg_bone_group: if flags.scales {
                                self.data
                                    .custom_skater
                                    .custom_classic
                                    .appearance
                                    .upper_leg_bone_group
                            } else {
                                Default::default()
                            },
                        },
                    },
                },
                story_skater: StorySkater {
                    tricks: if flags.trickset {
                        self.data.story_skater.tricks
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
        match save {
            Save::Thaw(save) => Ok(Self {
                summary: Summary::try_from(&save.summary)?,
                data: Data::try_from(&save.data)?,
            }),
            Save::Rethawed(save) => Ok(Self {
                summary: Summary::try_from(&save.try_get_chunk(chunk::Magic::Summary)?.structure)?,
                data: Data::try_from(&save.try_get_chunk(chunk::Magic::Data)?.structure)?,
            }),
        }
    }
}

impl Cas {
    pub fn modify(&self, save: &mut Save) -> cascade_core::Result<()> {
        self.summary.modify(match save {
            Save::Thaw(save) => &mut save.summary,
            Save::Rethawed(save) => &mut save.try_get_chunk_mut(chunk::Magic::Summary)?.structure,
        });
        self.data.modify(match save {
            Save::Thaw(save) => &mut save.data,
            Save::Rethawed(save) => &mut save.try_get_chunk_mut(chunk::Magic::Data)?.structure,
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Summary {
    pub filename: Item,
}

impl Summary {
    pub fn modify(&self, summary: &mut qb::Structure) {
        self.filename.modify(summary, id::FILENAME);
    }
}

impl TryFrom<&qb::Structure> for Summary {
    type Error = cascade_core::Error;

    fn try_from(summary: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            filename: summary.get(id::FILENAME).cloned().into(),
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Data {
    pub custom_skater: CustomSkater,
    pub story_skater: StorySkater,
}

impl TryFrom<&qb::Structure> for Data {
    type Error = cascade_core::Error;

    fn try_from(data: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            custom_skater: CustomSkater::try_from(expect_structure(data, id::CUSTOM_SKATER)?)?,
            story_skater: StorySkater::try_from(expect_structure(data, id::STORY_SKATER)?)?,
        })
    }
}

impl Data {
    pub fn modify(&self, data: &mut qb::Structure) -> cascade_core::Result<()> {
        self.custom_skater
            .modify(expect_structure_mut(data, id::CUSTOM_SKATER)?)?;

        self.story_skater
            .modify(expect_structure_mut(data, id::STORY_SKATER)?);

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomSkater {
    pub custom_classic: CustomClassic,
}

impl TryFrom<&qb::Structure> for CustomSkater {
    type Error = cascade_core::Error;

    fn try_from(custom_skater: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            custom_classic: CustomClassic::try_from(expect_structure(
                custom_skater,
                id::CUSTOM_CLASSIC,
            )?)?,
        })
    }
}

impl CustomSkater {
    pub fn modify(&self, custom_skater: &mut qb::Structure) -> cascade_core::Result<()> {
        self.custom_classic
            .modify(expect_structure_mut(custom_skater, id::CUSTOM_CLASSIC)?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomClassic {
    pub appearance: Appearance,
    pub info: Info,
}

impl TryFrom<&qb::Structure> for CustomClassic {
    type Error = cascade_core::Error;

    fn try_from(custom: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            appearance: Appearance::try_from(expect_structure(custom, id::APPEARANCE)?)?,
            info: Info::try_from(expect_structure(custom, id::INFO)?)?,
        })
    }
}

impl CustomClassic {
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
    pub specials: Item,
}

impl TryFrom<&qb::Structure> for Info {
    type Error = cascade_core::Error;

    fn try_from(info: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            trick_mapping: info.get(id::TRICK_MAPPING).cloned().into(),
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
    pub board_bone_group: Item,
    pub feet_bone_group: Item,
    pub hands_bone_group: Item,
    pub head_bone_group: Item,
    pub headtop_bone_group: Item,
    pub jaw_bone_group: Item,
    pub lower_arm_bone_group: Item,
    pub lower_leg_bone_group: Item,
    pub nose_bone_group: Item,
    pub object_scaling: Item,
    pub stomach_bone_group: Item,
    pub torso_bone_group: Item,
    pub upper_arm_bone_group: Item,
    pub upper_leg_bone_group: Item,
}

impl Appearance {
    pub fn modify(&self, appearance: &mut qb::Structure) {
        self.board_bone_group
            .modify(appearance, id::BOARD_BONE_GROUP);
        self.feet_bone_group.modify(appearance, id::FEET_BONE_GROUP);
        self.hands_bone_group
            .modify(appearance, id::HANDS_BONE_GROUP);
        self.head_bone_group.modify(appearance, id::HEAD_BONE_GROUP);
        self.headtop_bone_group
            .modify(appearance, id::HEADTOP_BONE_GROUP);
        self.jaw_bone_group.modify(appearance, id::JAW_BONE_GROUP);
        self.lower_arm_bone_group
            .modify(appearance, id::LOWER_ARM_BONE_GROUP);
        self.lower_leg_bone_group
            .modify(appearance, id::LOWER_LEG_BONE_GROUP);
        self.nose_bone_group.modify(appearance, id::NOSE_BONE_GROUP);
        self.object_scaling.modify(appearance, id::OBJECT_SCALING);
        self.stomach_bone_group
            .modify(appearance, id::STOMACH_BONE_GROUP);
        self.torso_bone_group
            .modify(appearance, id::TORSO_BONE_GROUP);
        self.upper_arm_bone_group
            .modify(appearance, id::UPPER_ARM_BONE_GROUP);
        self.upper_leg_bone_group
            .modify(appearance, id::UPPER_LEG_BONE_GROUP);
    }
}

impl TryFrom<&qb::Structure> for Appearance {
    type Error = cascade_core::Error;

    fn try_from(structure: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            board_bone_group: structure.get(id::BOARD_BONE_GROUP).cloned().into(),
            feet_bone_group: structure.get(id::FEET_BONE_GROUP).cloned().into(),
            hands_bone_group: structure.get(id::HANDS_BONE_GROUP).cloned().into(),
            head_bone_group: structure.get(id::HEAD_BONE_GROUP).cloned().into(),
            headtop_bone_group: structure.get(id::HEADTOP_BONE_GROUP).cloned().into(),
            jaw_bone_group: structure.get(id::JAW_BONE_GROUP).cloned().into(),
            lower_arm_bone_group: structure.get(id::LOWER_ARM_BONE_GROUP).cloned().into(),
            lower_leg_bone_group: structure.get(id::LOWER_LEG_BONE_GROUP).cloned().into(),
            nose_bone_group: structure.get(id::NOSE_BONE_GROUP).cloned().into(),
            object_scaling: structure.get(id::OBJECT_SCALING).cloned().into(),
            stomach_bone_group: structure.get(id::STOMACH_BONE_GROUP).cloned().into(),
            torso_bone_group: structure.get(id::TORSO_BONE_GROUP).cloned().into(),
            upper_arm_bone_group: structure.get(id::UPPER_ARM_BONE_GROUP).cloned().into(),
            upper_leg_bone_group: structure.get(id::UPPER_LEG_BONE_GROUP).cloned().into(),
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorySkater {
    pub tricks: Item,
}

impl TryFrom<&qb::Structure> for StorySkater {
    type Error = cascade_core::Error;

    fn try_from(structure: &qb::Structure) -> cascade_core::Result<Self> {
        Ok(Self {
            tricks: structure.get(id::TRICKS).cloned().into(),
        })
    }
}

impl StorySkater {
    pub fn modify(&self, story_skater: &mut qb::Structure) {
        self.tricks.modify(story_skater, id::TRICKS);
    }
}
