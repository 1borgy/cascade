use std::result;

use serde::Serialize;

use crate::{qb, save};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("qb error: {0}")]
    Qb(#[from] qb::Error),

    #[error("symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("symbol not found: {0}")]
    ExpectedStructure(String, qb::Value),
}

pub type Result<T, E = Error> = result::Result<T, E>;

fn expect_symbol(
    parent: Box<qb::Structure>,
    id: qb::Id,
    name: impl ToString,
) -> Result<qb::Symbol> {
    Ok(parent
        .get(id)
        .ok_or(Error::SymbolNotFound(name.to_string()))?
        .clone())
}

fn expect_symbol_mut(
    parent: &mut Box<qb::Structure>,
    id: qb::Id,
    name: impl ToString,
) -> Result<&mut qb::Symbol> {
    Ok(parent
        .get_mut(id)
        .ok_or(Error::SymbolNotFound(name.to_string()))?)
}

// expect symbol and expect structure
fn expect_structure(
    parent: Box<qb::Structure>,
    id: qb::Id,
    name: impl ToString,
) -> Result<Box<qb::Structure>> {
    let symbol = expect_symbol(parent, id, name)?;
    Ok(symbol.value.try_as_structure()?)
}

fn expect_structure_mut(
    parent: &mut Box<qb::Structure>,
    id: qb::Id,
    name: impl ToString,
) -> Result<&mut Box<qb::Structure>> {
    let symbol = expect_symbol_mut(parent, id, name)?;
    Ok(symbol.value.try_as_structure_mut()?)
}

#[derive(Debug, Clone, Serialize)]
pub struct Cas {
    pub header: save::Header,
    pub summary: Summary,
    pub data: Data,
}

impl TryFrom<save::Content> for Cas {
    type Error = Error;

    fn try_from(content: save::Content) -> Result<Self> {
        Ok(Self {
            header: content.header,
            data: Data::try_from(content.data)?,
            summary: Summary(content.summary),
        })
    }
}

impl Cas {
    pub fn modify(&self, content: &mut save::Content) -> Result<()> {
        self.data.modify(&mut content.data)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary(pub Box<qb::Structure>);

const CUSTOM_SKATER_NAME: &'static str = "CustomSkater";
const CUSTOM_SKATER_ID: qb::Id = qb::Id::Checksum(314551426);

const STORY_SKATER_NAME: &'static str = "StorySkater";
const STORY_SKATER_ID: qb::Id = qb::Id::Checksum(234026056);

#[derive(Debug, Clone, Serialize, Default)]
pub struct Data {
    pub custom_skater: Option<CustomSkater>,
    pub story_skater: Option<StorySkater>,
}

impl TryFrom<Box<qb::Structure>> for Data {
    type Error = Error;

    fn try_from(data: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            custom_skater: Some(CustomSkater::try_from(expect_structure(
                Box::clone(&data),
                CUSTOM_SKATER_ID,
                CUSTOM_SKATER_NAME,
            )?)?),
            story_skater: Some(StorySkater::try_from(expect_structure(
                Box::clone(&data),
                STORY_SKATER_ID,
                STORY_SKATER_NAME,
            )?)?),
        })
    }
}

impl Data {
    pub fn modify(&self, data: &mut Box<qb::Structure>) -> Result<()> {
        if let Some(custom_skater) = &self.custom_skater {
            custom_skater.modify(expect_structure_mut(
                data,
                CUSTOM_SKATER_ID,
                CUSTOM_SKATER_NAME,
            )?)?;
        }

        if let Some(story_skater) = &self.story_skater {
            story_skater.modify(expect_structure_mut(
                data,
                STORY_SKATER_ID,
                STORY_SKATER_NAME,
            )?)?;
        }

        Ok(())
    }
}

const CUSTOM_NAME: &'static str = "CustomSkater.custom";
const CUSTOM_ID: qb::Id = qb::Id::Compressed8(195);

#[derive(Debug, Clone, Serialize, Default)]
pub struct CustomSkater {
    pub custom: Option<Custom>,
}

impl TryFrom<Box<qb::Structure>> for CustomSkater {
    type Error = Error;

    fn try_from(custom_skater: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            custom: Some(Custom::try_from(expect_structure(
                custom_skater,
                CUSTOM_ID,
                CUSTOM_NAME,
            )?)?),
        })
    }
}

impl CustomSkater {
    pub fn modify(&self, custom_skater: &mut Box<qb::Structure>) -> Result<()> {
        if let Some(custom) = &self.custom {
            custom.modify(expect_structure_mut(
                custom_skater,
                CUSTOM_ID,
                CUSTOM_NAME,
            )?)?;
        }
        Ok(())
    }
}

const APPEARANCE_NAME: &'static str = "CustomSkater.custom.appearance";
const APPEARANCE_ID: qb::Id = qb::Id::Checksum(1431076207);

const INFO_NAME: &'static str = "CustomSkater.custom.info";
const INFO_ID: qb::Id = qb::Id::Checksum(880201384);

// TODO: implement entire Appearance struct
#[derive(Debug, Clone, Serialize, Default)]
pub struct Custom {
    pub scales: Option<Scales>,
    pub info: Option<Info>,
}

impl TryFrom<Box<qb::Structure>> for Custom {
    type Error = Error;

    fn try_from(custom: Box<qb::Structure>) -> Result<Self> {
        let appearance = expect_structure(
            Box::clone(&custom),
            APPEARANCE_ID,
            APPEARANCE_NAME,
        )?;

        Ok(Self {
            scales: Some(Scales::try_from(appearance)?),
            info: Some(Info::try_from(expect_structure(
                custom, INFO_ID, INFO_NAME,
            )?)?),
        })
    }
}

impl Custom {
    pub fn modify(&self, custom: &mut Box<qb::Structure>) -> Result<()> {
        if let Some(scales) = &self.scales {
            let appearance =
                expect_structure_mut(custom, APPEARANCE_ID, APPEARANCE_NAME)?;

            scales.modify(appearance);
        }

        if let Some(info) = &self.info {
            info.modify(expect_structure_mut(custom, INFO_ID, INFO_NAME)?)?;
        }

        Ok(())
    }
}

const TRICK_MAPPING_NAME: &'static str =
    "CustomSkater.custom.info.trick_mapping";
const TRICK_MAPPING_ID: qb::Id = qb::Id::Compressed8(61);

const SPECIALS_NAME: &'static str = "CustomSkater.custom.info.specials";
const SPECIALS_ID: qb::Id = qb::Id::Compressed8(64);

#[derive(Debug, Clone, Serialize, Default)]
pub struct Info {
    pub trick_mapping: Option<qb::Symbol>,
    pub specials: Option<qb::Symbol>,
}

impl TryFrom<Box<qb::Structure>> for Info {
    type Error = Error;

    fn try_from(info: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            trick_mapping: Some(expect_symbol(
                Box::clone(&info),
                TRICK_MAPPING_ID,
                TRICK_MAPPING_NAME,
            )?),
            specials: Some(expect_symbol(
                Box::clone(&info),
                SPECIALS_ID,
                SPECIALS_NAME,
            )?),
        })
    }
}

impl Info {
    pub fn modify(&self, info: &mut qb::Structure) -> Result<()> {
        if let Some(trick_mapping) = &self.trick_mapping {
            info.insert(trick_mapping.clone());
        }

        if let Some(specials) = &self.specials {
            info.insert(specials.clone());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Scales {
    pub board_bone_group: Scalable,
    pub feet_bone_group: Scalable,
    pub hands_bone_group: Scalable,
    pub head_bone_group: Scalable,
    pub headtop_bone_group: Scalable,
    pub jaw_bone_group: Scalable,
    pub lower_arm_bone_group: Scalable,
    pub lower_leg_bone_group: Scalable,
    pub nose_bone_group: Scalable,
    pub object_scaling: Scalable,
    pub stomach_bone_group: Scalable,
    pub torso_bone_group: Scalable,
    pub upper_arm_bone_group: Scalable,
    pub upper_leg_bone_group: Scalable,
}

impl Scales {
    pub fn modify(&self, appearance: &mut qb::Structure) {
        self.board_bone_group
            .modify(appearance, BOARD_BONE_GROUP_ID);
        self.feet_bone_group.modify(appearance, FEET_BONE_GROUP_ID);
        self.hands_bone_group
            .modify(appearance, HANDS_BONE_GROUP_ID);
        self.head_bone_group.modify(appearance, HEAD_BONE_GROUP_ID);
        self.headtop_bone_group
            .modify(appearance, HEADTOP_BONE_GROUP_ID);
        self.jaw_bone_group.modify(appearance, JAW_BONE_GROUP_ID);
        self.lower_arm_bone_group
            .modify(appearance, LOWER_ARM_BONE_GROUP_ID);
        self.lower_leg_bone_group
            .modify(appearance, LOWER_LEG_BONE_GROUP_ID);
        self.nose_bone_group.modify(appearance, NOSE_BONE_GROUP_ID);
        self.object_scaling.modify(appearance, OBJECT_SCALING_ID);
        self.stomach_bone_group
            .modify(appearance, STOMACH_BONE_GROUP_ID);
        self.torso_bone_group
            .modify(appearance, TORSO_BONE_GROUP_ID);
        self.upper_arm_bone_group
            .modify(appearance, UPPER_ARM_BONE_GROUP_ID);
        self.upper_leg_bone_group
            .modify(appearance, UPPER_LEG_BONE_GROUP_ID);
    }
}

const BOARD_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(208);
const FEET_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(207);
const HANDS_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(204);
const HEAD_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(199);
const HEADTOP_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(196);
const JAW_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(197);
const LOWER_ARM_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(203);
const LOWER_LEG_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(206);
const NOSE_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(198);
const OBJECT_SCALING_ID: qb::Id = qb::Id::Compressed8(209);
const STOMACH_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(201);
const TORSO_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(200);
const UPPER_ARM_BONE_GROUP_ID: qb::Id = qb::Id::Compressed8(202);
const UPPER_LEG_BONE_GROUP_ID: qb::Id = qb::Id::Checksum(3191687513);

impl TryFrom<Box<qb::Structure>> for Scales {
    type Error = Error;

    fn try_from(structure: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            board_bone_group: Scalable::from(
                structure.get(BOARD_BONE_GROUP_ID).cloned(),
            ),
            feet_bone_group: Scalable::from(
                structure.get(FEET_BONE_GROUP_ID).cloned(),
            ),
            hands_bone_group: Scalable::from(
                structure.get(HANDS_BONE_GROUP_ID).cloned(),
            ),
            head_bone_group: Scalable::from(
                structure.get(HEAD_BONE_GROUP_ID).cloned(),
            ),
            headtop_bone_group: Scalable::from(
                structure.get(HEADTOP_BONE_GROUP_ID).cloned(),
            ),
            jaw_bone_group: Scalable::from(
                structure.get(JAW_BONE_GROUP_ID).cloned(),
            ),
            lower_arm_bone_group: Scalable::from(
                structure.get(LOWER_ARM_BONE_GROUP_ID).cloned(),
            ),
            lower_leg_bone_group: Scalable::from(
                structure.get(LOWER_LEG_BONE_GROUP_ID).cloned(),
            ),
            nose_bone_group: Scalable::from(
                structure.get(NOSE_BONE_GROUP_ID).cloned(),
            ),
            object_scaling: Scalable::from(
                structure.get(OBJECT_SCALING_ID).cloned(),
            ),
            stomach_bone_group: Scalable::from(
                structure.get(STOMACH_BONE_GROUP_ID).cloned(),
            ),
            torso_bone_group: Scalable::from(
                structure.get(TORSO_BONE_GROUP_ID).cloned(),
            ),
            upper_arm_bone_group: Scalable::from(
                structure.get(UPPER_ARM_BONE_GROUP_ID).cloned(),
            ),
            upper_leg_bone_group: Scalable::from(
                structure.get(UPPER_LEG_BONE_GROUP_ID).cloned(),
            ),
        })
    }
}

//const X_ID: qb::Id = qb::Id::Compressed8(165);
//const Y_ID: qb::Id = qb::Id::Compressed8(166);
//const Z_ID: qb::Id = qb::Id::Compressed8(167);
//const USE_DEFAULT_SCALE_ID: qb::Id = qb::Id::Compressed8(26);

//#[derive(Debug, Clone, Serialize)]
//pub struct Scaled {
//    pub x: qb::Symbol,                 // "X"
//    pub y: qb::Symbol,                 // "Y"
//    pub z: qb::Symbol,                 // "Z"
//    pub use_default_scale: qb::Symbol, // "use_default_scale"
//}

//impl TryFrom<Box<qb::Structure>> for Scaled {
//    type Error = Error;
//    fn try_from(structure: Box<qb::Structure>) -> Result<Self> {
//        let x = expect_symbol(Box::clone(&structure), X_ID, "{Scaled}.X")?;
//        let y = expect_symbol(Box::clone(&structure), Y_ID, "{Scaled}.Y")?;
//        let z = expect_symbol(Box::clone(&structure), Z_ID, "{Scaled}.Z")?;
//        let use_default_scale = expect_symbol(
//            Box::clone(&structure),
//            USE_DEFAULT_SCALE_ID,
//            "{Scaled}.Z",
//        )?;
//
//        Ok(Self {
//            x,
//            y,
//            z,
//            use_default_scale,
//        })
//    }
//}

// have not observed a scaled item only have desc_id, unlike colors
#[derive(Debug, Clone, Serialize)]
pub enum Scalable {
    Scaled(qb::Symbol),
    Vacant,
}

impl Scalable {
    pub fn modify(&self, appearance: &mut qb::Structure, id: qb::Id) {
        match self {
            Scalable::Scaled(symbol) => {
                appearance.insert(symbol.clone());
            }
            Scalable::Vacant => {
                appearance.remove(id);
            }
        }
    }
}

impl From<Option<qb::Symbol>> for Scalable {
    fn from(structure: Option<qb::Symbol>) -> Self {
        match structure {
            Some(structure) => Self::Scaled(structure),
            None => Self::Vacant,
        }
    }
}

//#[derive(Debug)]
//struct Colored {
//    desc_id: Option<qb::Symbol>,
//    // lowercase hsv
//    h: Option<qb::Symbol>,
//    s: Option<qb::Symbol>,
//    v: Option<qb::Symbol>,
//    use_default_hsv: Option<qb::Symbol>, // ZeroInt
//}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StorySkater {
    // qb::Value::Array(qb::Kind::Structure, ...)
    pub tricks: Option<qb::Symbol>,
}

const TRICKS_ID: qb::Id = qb::Id::Checksum(505871678);
const TRICKS_NAME: &'static str = "StorySkater.tricks";

impl TryFrom<Box<qb::Structure>> for StorySkater {
    type Error = Error;

    fn try_from(structure: Box<qb::Structure>) -> Result<Self> {
        Ok(Self {
            tricks: Some(expect_symbol(structure, TRICKS_ID, TRICKS_NAME)?),
        })
    }
}

impl StorySkater {
    pub fn modify(&self, story_skater: &mut Box<qb::Structure>) -> Result<()> {
        if let Some(tricks) = &self.tricks {
            story_skater.insert(tricks.clone());
        }

        Ok(())
    }
}
