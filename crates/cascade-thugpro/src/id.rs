use cascade_qb as qb;

pub const TOTAL_GOALS_COMPLETE: qb::Id = qb::Id::Checksum(2510147438);
pub const TOTAL_GOALS_POSSIBLE: qb::Id = qb::Id::Checksum(2841615987);
pub const TOTAL_SCORE: qb::Id = qb::Id::Checksum(236215964);
pub const TOTAL_SCORE_POTENTIAL: qb::Id = qb::Id::Checksum(2233141508);
pub const IS_MALE: qb::Id = qb::Id::Compress8(220);
pub const NAME: qb::Id = qb::Id::Compress8(43);
pub const FILENAME: qb::Id = qb::Id::Checksum(3287553690);

pub const CUSTOM_SKATER: qb::Id = qb::Id::Checksum(314551426);
pub const STORY_SKATER: qb::Id = qb::Id::Checksum(234026056);

pub const CUSTOM: qb::Id = qb::Id::Compress8(195);

pub const APPEARANCE: qb::Id = qb::Id::Checksum(1431076207);
pub const INFO: qb::Id = qb::Id::Checksum(880201384);

pub const TRICK_MAPPING: qb::Id = qb::Id::Compress8(61);
pub const SPECIALS: qb::Id = qb::Id::Compress8(64);

pub const BODY_SHAPE: qb::Id = qb::Id::Checksum(2166785263);
pub const BODY: qb::Id = qb::Id::Checksum(609743949);

pub const SKATER_M_HEAD: qb::Id = qb::Id::Compress8(1);
pub const SKATER_F_HEAD: qb::Id = qb::Id::Compress8(10);
pub const SKATER_M_HAIR: qb::Id = qb::Id::Compress8(4);
pub const SKATER_F_HAIR: qb::Id = qb::Id::Compress8(13);
pub const SKATER_M_HAT_HAIR: qb::Id = qb::Id::Compress8(16);
pub const SKATER_F_HAT_HAIR: qb::Id = qb::Id::Compress8(36);
pub const HAT: qb::Id = qb::Id::Compress8(180);
pub const HAT_LOGO: qb::Id = qb::Id::Compress8(183);
pub const EYES: qb::Id = qb::Id::Checksum(1491454825);
pub const GLASSES: qb::Id = qb::Id::Compress8(179);

pub const BARE_TORSO: qb::Id = qb::Id::Compress8(192);
pub const SKATER_M_TORSO: qb::Id = qb::Id::Compress8(2);
pub const SKATER_F_TORSO: qb::Id = qb::Id::Compress8(11);
pub const FRONT_LOGO: qb::Id = qb::Id::Compress8(177);
pub const BACK_LOGO: qb::Id = qb::Id::Compress8(178);
pub const SKATER_M_HANDS: qb::Id = qb::Id::Compress8(18);
pub const SKATER_F_HANDS: qb::Id = qb::Id::Compress8(38);
pub const ACCESSORY1: qb::Id = qb::Id::Compress8(188);
pub const ACCESSORY2: qb::Id = qb::Id::Compress8(189);
pub const ACCESSORY3: qb::Id = qb::Id::Compress8(190);
pub const ELBOWPADS: qb::Id = qb::Id::Compress8(9);
pub const SLEEVES: qb::Id = qb::Id::Compress8(175);
pub const SKATER_M_BACKPACK: qb::Id = qb::Id::Compress8(5);
pub const SKATER_F_BACKPACK: qb::Id = qb::Id::Compress8(14);

pub const SKATER_M_LEGS: qb::Id = qb::Id::Compress8(3);
pub const SKATER_F_LEGS: qb::Id = qb::Id::Compress8(12);
pub const SKATER_M_LOWER_LEGS: qb::Id = qb::Id::Compress8(0);
pub const SKATER_F_LOWER_LEGS: qb::Id = qb::Id::Compress8(35);
pub const KNEEPADS: qb::Id = qb::Id::Compress8(7);

pub const SHOES: qb::Id = qb::Id::Compress8(176);
pub const SOCKS: qb::Id = qb::Id::Compress8(8);
pub const SHOE_LACES: qb::Id = qb::Id::Compress8(191);

pub const BOARD: qb::Id = qb::Id::Compress8(185);
pub const DECK_GRAPHIC: qb::Id = qb::Id::Compress8(30);
pub const GRIPTAPE: qb::Id = qb::Id::Compress8(186);

pub const LEFT_SLEEVE_TATTOO: qb::Id = qb::Id::Checksum(1037744690);
pub const RIGHT_SLEEVE_TATTOO: qb::Id = qb::Id::Checksum(2633440348);
pub const LEFT_FOREARM_TATTOO: qb::Id = qb::Id::Compress8(21);
pub const RIGHT_FOREARM_TATTOO: qb::Id = qb::Id::Compress8(22);
pub const LEFT_BICEP_TATTOO: qb::Id = qb::Id::Compress8(23);
pub const RIGHT_BICEP_TATTOO: qb::Id = qb::Id::Compress8(27);
pub const BACK_TATTOO: qb::Id = qb::Id::Compress8(20);
pub const CHEST_TATTOO: qb::Id = qb::Id::Compress8(19);
pub const LEFT_LEG_TATTOO: qb::Id = qb::Id::Compress8(24);
pub const RIGHT_LEG_TATTOO: qb::Id = qb::Id::Compress8(25);

pub const BOARD_BONE_GROUP: qb::Id = qb::Id::Compress8(208);
pub const FEET_BONE_GROUP: qb::Id = qb::Id::Compress8(207);
pub const HANDS_BONE_GROUP: qb::Id = qb::Id::Compress8(204);
pub const HEAD_BONE_GROUP: qb::Id = qb::Id::Compress8(199);
pub const HEADTOP_BONE_GROUP: qb::Id = qb::Id::Compress8(196);
pub const JAW_BONE_GROUP: qb::Id = qb::Id::Compress8(197);
pub const LOWER_ARM_BONE_GROUP: qb::Id = qb::Id::Compress8(203);
pub const LOWER_LEG_BONE_GROUP: qb::Id = qb::Id::Compress8(206);
pub const NOSE_BONE_GROUP: qb::Id = qb::Id::Compress8(198);
pub const OBJECT_SCALING: qb::Id = qb::Id::Compress8(209);
pub const STOMACH_BONE_GROUP: qb::Id = qb::Id::Compress8(201);
pub const TORSO_BONE_GROUP: qb::Id = qb::Id::Compress8(200);
pub const UPPER_ARM_BONE_GROUP: qb::Id = qb::Id::Compress8(202);
pub const UPPER_LEG_BONE_GROUP: qb::Id = qb::Id::Checksum(3191687513);

pub const PED_M_ACCESSORIES: qb::Id = qb::Id::Checksum(3856410554);
pub const PED_F_ACCESSORIES: qb::Id = qb::Id::Checksum(34969625);

pub const TRICKS: qb::Id = qb::Id::Checksum(505871678);
