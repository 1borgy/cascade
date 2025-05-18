use cascade_qb as qb;

use crate::{Error, Result, id, save};

fn expect_symbol(parent: &Box<qb::Structure>, id: qb::Id) -> Result<qb::Symbol> {
    Ok(parent.get(id).ok_or(Error::SymbolNotFound(id))?.clone())
}

fn expect_symbol_mut(parent: &mut Box<qb::Structure>, id: qb::Id) -> Result<&mut qb::Symbol> {
    Ok(parent.get_mut(id).ok_or(Error::SymbolNotFound(id))?)
}

// expect symbol and expect structure
fn expect_structure(parent: Box<qb::Structure>, id: qb::Id) -> Result<Box<qb::Structure>> {
    let symbol = expect_symbol(&parent, id)?;
    Ok(symbol.value.try_as_structure()?)
}

fn expect_structure_mut(
    parent: &mut Box<qb::Structure>,
    id: qb::Id,
) -> Result<&mut Box<qb::Structure>> {
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
    pub summary: Summary,
    pub data: Data,
}

impl TryFrom<save::Save> for Cas {
    type Error = Error;

    fn try_from(save: save::Save) -> Result<Self> {
        Ok(Self {
            summary: Summary::try_from(save.summary)?,
            data: Data::try_from(save.data)?,
        })
    }
}

impl Cas {
    pub fn modify(&self, save: &mut save::Save) -> Result<()> {
        self.summary.modify(&mut save.summary);
        self.data.modify(&mut save.data)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Summary {
    pub total_goals_complete: Item,
    pub total_goals_possible: Item,
    pub total_score: Item,
    pub total_score_potential: Item,
    pub is_male: Item,
    pub name: Item,
    pub filename: Item,
}

impl Summary {
    pub fn modify(&self, summary: &mut Box<qb::Structure>) {
        self.total_goals_complete
            .modify(summary, id::TOTAL_GOALS_COMPLETE);
        self.total_goals_possible
            .modify(summary, id::TOTAL_GOALS_POSSIBLE);
        self.total_score.modify(summary, id::TOTAL_SCORE);
        self.total_score_potential
            .modify(summary, id::TOTAL_SCORE_POTENTIAL);
        self.is_male.modify(summary, id::IS_MALE);
        self.name.modify(summary, id::NAME);
        self.filename.modify(summary, id::FILENAME);
    }
}

impl TryFrom<Box<qb::Structure>> for Summary {
    type Error = Error;

    fn try_from(summary: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            total_goals_complete: summary.get(id::TOTAL_GOALS_COMPLETE).cloned().into(),
            total_goals_possible: summary.get(id::TOTAL_GOALS_POSSIBLE).cloned().into(),
            total_score: summary.get(id::TOTAL_SCORE).cloned().into(),
            total_score_potential: summary.get(id::TOTAL_SCORE_POTENTIAL).cloned().into(),
            is_male: summary.get(id::IS_MALE).cloned().into(),
            name: summary.get(id::NAME).cloned().into(),
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

impl TryFrom<Box<qb::Structure>> for Data {
    type Error = Error;

    fn try_from(data: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            custom_skater: CustomSkater::try_from(expect_structure(
                Box::clone(&data),
                id::CUSTOM_SKATER,
            )?)?,
            story_skater: StorySkater::try_from(expect_structure(
                Box::clone(&data),
                id::STORY_SKATER,
            )?)?,
        })
    }
}

impl Data {
    pub fn modify(&self, data: &mut Box<qb::Structure>) -> Result<()> {
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
    pub custom: Custom,
}

impl TryFrom<Box<qb::Structure>> for CustomSkater {
    type Error = Error;

    fn try_from(custom_skater: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            custom: Custom::try_from(expect_structure(custom_skater, id::CUSTOM)?)?,
        })
    }
}

impl CustomSkater {
    pub fn modify(&self, custom_skater: &mut Box<qb::Structure>) -> Result<()> {
        self.custom
            .modify(expect_structure_mut(custom_skater, id::CUSTOM)?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Custom {
    pub appearance: Appearance,
    pub info: Info,
}

impl TryFrom<Box<qb::Structure>> for Custom {
    type Error = Error;

    fn try_from(custom: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            appearance: Appearance::try_from(expect_structure(
                Box::clone(&custom),
                id::APPEARANCE,
            )?)?,
            info: Info::try_from(expect_structure(custom, id::INFO)?)?,
        })
    }
}

impl Custom {
    pub fn modify(&self, custom: &mut Box<qb::Structure>) -> Result<()> {
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

impl TryFrom<Box<qb::Structure>> for Info {
    type Error = Error;

    fn try_from(info: Box<qb::Structure>) -> Result<Self> {
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
    pub body_shape: Item,
    pub body: Item,

    pub skater_m_head: Item,
    pub skater_f_head: Item,
    pub skater_m_hair: Item,
    pub skater_f_hair: Item,
    pub skater_m_hat_hair: Item,
    pub skater_f_hat_hair: Item,
    pub hat: Item,
    pub hat_logo: Item,
    pub eyes: Item,
    pub glasses: Item,

    pub bare_torso: Item,
    pub skater_m_torso: Item,
    pub skater_f_torso: Item,
    pub front_logo: Item,
    pub back_logo: Item,
    pub skater_m_hands: Item,
    pub skater_f_hands: Item,
    pub accessory1: Item,
    pub accessory2: Item,
    pub accessory3: Item,
    pub elbowpads: Item,
    pub sleeves: Item,

    pub skater_m_backpack: Item,
    pub skater_f_backpack: Item,

    pub skater_m_legs: Item,
    pub skater_f_legs: Item,
    pub skater_m_lower_legs: Item,
    pub skater_f_lower_legs: Item,
    pub kneepads: Item,

    pub shoes: Item,
    pub socks: Item,
    pub shoe_laces: Item,

    pub board: Item,
    pub deck_graphic: Item,
    pub griptape: Item,

    pub left_sleeve_tattoo: Item,
    pub right_sleeve_tattoo: Item,
    pub left_forearm_tattoo: Item,
    pub right_forearm_tattoo: Item,
    pub left_bicep_tattoo: Item,
    pub right_bicep_tattoo: Item,
    pub back_tattoo: Item,
    pub chest_tattoo: Item,
    pub left_leg_tattoo: Item,
    pub right_leg_tattoo: Item,

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

    pub ped_m_accessories: Item,
    pub ped_f_accessories: Item,
}

impl Appearance {
    pub fn modify(&self, appearance: &mut qb::Structure) {
        self.body_shape.modify(appearance, id::BODY_SHAPE);
        self.body.modify(appearance, id::BODY);
        self.skater_m_head.modify(appearance, id::SKATER_M_HEAD);
        self.skater_f_head.modify(appearance, id::SKATER_F_HEAD);
        self.skater_m_hair.modify(appearance, id::SKATER_M_HAIR);
        self.skater_f_hair.modify(appearance, id::SKATER_F_HAIR);
        self.skater_m_hat_hair
            .modify(appearance, id::SKATER_M_HAT_HAIR);
        self.skater_f_hat_hair
            .modify(appearance, id::SKATER_F_HAT_HAIR);
        self.hat.modify(appearance, id::HAT);
        self.hat_logo.modify(appearance, id::HAT_LOGO);
        self.eyes.modify(appearance, id::EYES);
        self.glasses.modify(appearance, id::GLASSES);
        self.bare_torso.modify(appearance, id::BARE_TORSO);
        self.skater_m_torso.modify(appearance, id::SKATER_M_TORSO);
        self.skater_f_torso.modify(appearance, id::SKATER_F_TORSO);
        self.front_logo.modify(appearance, id::FRONT_LOGO);
        self.back_logo.modify(appearance, id::BACK_LOGO);
        self.skater_m_hands.modify(appearance, id::SKATER_M_HANDS);
        self.skater_f_hands.modify(appearance, id::SKATER_F_HANDS);
        self.accessory1.modify(appearance, id::ACCESSORY1);
        self.accessory2.modify(appearance, id::ACCESSORY2);
        self.accessory3.modify(appearance, id::ACCESSORY3);
        self.elbowpads.modify(appearance, id::ELBOWPADS);
        self.sleeves.modify(appearance, id::SLEEVES);
        self.skater_m_backpack
            .modify(appearance, id::SKATER_M_BACKPACK);
        self.skater_f_backpack
            .modify(appearance, id::SKATER_F_BACKPACK);
        self.skater_m_legs.modify(appearance, id::SKATER_M_LEGS);
        self.skater_f_legs.modify(appearance, id::SKATER_F_LEGS);
        self.skater_m_lower_legs
            .modify(appearance, id::SKATER_M_LOWER_LEGS);
        self.skater_f_lower_legs
            .modify(appearance, id::SKATER_F_LOWER_LEGS);
        self.kneepads.modify(appearance, id::KNEEPADS);
        self.shoes.modify(appearance, id::SHOES);
        self.socks.modify(appearance, id::SOCKS);
        self.shoe_laces.modify(appearance, id::SHOE_LACES);
        self.board.modify(appearance, id::BOARD);
        self.deck_graphic.modify(appearance, id::DECK_GRAPHIC);
        self.griptape.modify(appearance, id::GRIPTAPE);
        self.left_sleeve_tattoo
            .modify(appearance, id::LEFT_SLEEVE_TATTOO);
        self.right_sleeve_tattoo
            .modify(appearance, id::RIGHT_SLEEVE_TATTOO);
        self.left_forearm_tattoo
            .modify(appearance, id::LEFT_FOREARM_TATTOO);
        self.right_forearm_tattoo
            .modify(appearance, id::RIGHT_FOREARM_TATTOO);
        self.left_bicep_tattoo
            .modify(appearance, id::LEFT_BICEP_TATTOO);
        self.right_bicep_tattoo
            .modify(appearance, id::RIGHT_BICEP_TATTOO);
        self.back_tattoo.modify(appearance, id::BACK_TATTOO);
        self.chest_tattoo.modify(appearance, id::CHEST_TATTOO);
        self.left_leg_tattoo.modify(appearance, id::LEFT_LEG_TATTOO);
        self.right_leg_tattoo
            .modify(appearance, id::RIGHT_LEG_TATTOO);
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
        self.ped_m_accessories
            .modify(appearance, id::PED_M_ACCESSORIES);
        self.ped_f_accessories
            .modify(appearance, id::PED_F_ACCESSORIES);
    }
}

impl TryFrom<Box<qb::Structure>> for Appearance {
    type Error = Error;

    fn try_from(structure: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            body_shape: structure.get(id::BODY_SHAPE).cloned().into(),
            body: structure.get(id::BODY).cloned().into(),
            skater_m_head: structure.get(id::SKATER_M_HEAD).cloned().into(),
            skater_f_head: structure.get(id::SKATER_F_HEAD).cloned().into(),
            skater_m_hair: structure.get(id::SKATER_M_HAIR).cloned().into(),
            skater_f_hair: structure.get(id::SKATER_F_HAIR).cloned().into(),
            skater_m_hat_hair: structure.get(id::SKATER_M_HAT_HAIR).cloned().into(),
            skater_f_hat_hair: structure.get(id::SKATER_F_HAT_HAIR).cloned().into(),
            hat: structure.get(id::HAT).cloned().into(),
            hat_logo: structure.get(id::HAT_LOGO).cloned().into(),
            eyes: structure.get(id::EYES).cloned().into(),
            glasses: structure.get(id::GLASSES).cloned().into(),
            bare_torso: structure.get(id::BARE_TORSO).cloned().into(),
            skater_m_torso: structure.get(id::SKATER_M_TORSO).cloned().into(),
            skater_f_torso: structure.get(id::SKATER_F_TORSO).cloned().into(),
            front_logo: structure.get(id::FRONT_LOGO).cloned().into(),
            back_logo: structure.get(id::BACK_LOGO).cloned().into(),
            skater_m_hands: structure.get(id::SKATER_M_HANDS).cloned().into(),
            skater_f_hands: structure.get(id::SKATER_F_HANDS).cloned().into(),
            accessory1: structure.get(id::ACCESSORY1).cloned().into(),
            accessory2: structure.get(id::ACCESSORY2).cloned().into(),
            accessory3: structure.get(id::ACCESSORY3).cloned().into(),
            elbowpads: structure.get(id::ELBOWPADS).cloned().into(),
            sleeves: structure.get(id::SLEEVES).cloned().into(),
            skater_m_backpack: structure.get(id::SKATER_M_BACKPACK).cloned().into(),
            skater_f_backpack: structure.get(id::SKATER_F_BACKPACK).cloned().into(),
            skater_m_legs: structure.get(id::SKATER_M_LEGS).cloned().into(),
            skater_f_legs: structure.get(id::SKATER_F_LEGS).cloned().into(),
            skater_m_lower_legs: structure.get(id::SKATER_M_LOWER_LEGS).cloned().into(),
            skater_f_lower_legs: structure.get(id::SKATER_F_LOWER_LEGS).cloned().into(),
            kneepads: structure.get(id::KNEEPADS).cloned().into(),
            shoes: structure.get(id::SHOES).cloned().into(),
            socks: structure.get(id::SOCKS).cloned().into(),
            shoe_laces: structure.get(id::SHOE_LACES).cloned().into(),
            board: structure.get(id::BOARD).cloned().into(),
            deck_graphic: structure.get(id::DECK_GRAPHIC).cloned().into(),
            griptape: structure.get(id::GRIPTAPE).cloned().into(),
            left_sleeve_tattoo: structure.get(id::LEFT_SLEEVE_TATTOO).cloned().into(),
            right_sleeve_tattoo: structure.get(id::RIGHT_SLEEVE_TATTOO).cloned().into(),
            left_forearm_tattoo: structure.get(id::LEFT_FOREARM_TATTOO).cloned().into(),
            right_forearm_tattoo: structure.get(id::RIGHT_FOREARM_TATTOO).cloned().into(),
            left_bicep_tattoo: structure.get(id::LEFT_BICEP_TATTOO).cloned().into(),
            right_bicep_tattoo: structure.get(id::RIGHT_BICEP_TATTOO).cloned().into(),
            back_tattoo: structure.get(id::BACK_TATTOO).cloned().into(),
            chest_tattoo: structure.get(id::CHEST_TATTOO).cloned().into(),
            left_leg_tattoo: structure.get(id::LEFT_LEG_TATTOO).cloned().into(),
            right_leg_tattoo: structure.get(id::RIGHT_LEG_TATTOO).cloned().into(),
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
            ped_m_accessories: structure.get(id::PED_M_ACCESSORIES).cloned().into(),
            ped_f_accessories: structure.get(id::PED_F_ACCESSORIES).cloned().into(),
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorySkater {
    pub tricks: Item,
}

impl TryFrom<Box<qb::Structure>> for StorySkater {
    type Error = Error;

    fn try_from(structure: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            tricks: structure.get(id::TRICKS).cloned().into(),
        })
    }
}

impl StorySkater {
    pub fn modify(&self, story_skater: &mut Box<qb::Structure>) {
        self.tricks.modify(story_skater, id::TRICKS);
    }
}
