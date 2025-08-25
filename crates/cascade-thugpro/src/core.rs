use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use cascade_core as core;
use cascade_save::Save;

use crate::{
    Error,
    cas::{self, Cas},
    entry::{self, Entry},
};

pub struct Explorer {
    cwd: PathBuf,
}

impl Explorer {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

impl core::Explorer<Entry, Error> for Explorer {
    fn list(&self) -> Result<impl Iterator<Item = Entry>, Error> {
        entry::find_entries(&self.cwd).map(|entries| entries.into_iter())
    }

    fn reader(&self, entry: &Entry) -> Result<impl Read + Seek, Error> {
        entry.reader()
    }

    fn writer(&self, entry: &Entry) -> Result<impl std::io::Write, Error> {
        entry.writer()
    }

    fn name(&self, entry: &Entry) -> String {
        entry.filename.clone()
    }

    fn rewrite_metadata(&self, entry: &Entry) -> Result<(), Error> {
        entry.rewrite_metadata()
    }
}

pub struct Parser;

impl core::Parser<Save, Cas, Error> for Parser {
    fn read(&self, reader: &mut impl Read) -> Result<Save, Error> {
        Ok(Save::read(reader)?)
    }

    fn write(&self, save: &Save, writer: &mut impl std::io::Write) -> Result<(), Error> {
        Ok(save.write(writer)?)
    }

    fn parse(&self, save: &Save) -> Result<Cas, Error> {
        Ok(Cas::try_from(save)?)
    }

    fn modify(&self, save: &mut Save, transform: &Cas) -> Result<(), Error> {
        Ok(transform.modify(save)?)
    }

    fn mask(&self, cas: Cas, flags: core::Flags) -> Cas {
        Cas {
            summary: flags.summary.then_some(cas.summary).unwrap_or_default(),
            data: cas::Data {
                custom_skater: cas::CustomSkater {
                    custom: cas::Custom {
                        info: cas::Info {
                            trick_mapping: flags
                                .trickset
                                .then_some(cas.data.custom_skater.custom.info.trick_mapping)
                                .unwrap_or_default(),
                            specials: flags
                                .trickset
                                .then_some(cas.data.custom_skater.custom.info.specials)
                                .unwrap_or_default(),
                        },
                        appearance: cas::Appearance {
                            board_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data.custom_skater.custom.appearance.board_bone_group,
                                )
                                .unwrap_or_default(),
                            feet_bone_group: flags
                                .scales
                                .then_some(cas.data.custom_skater.custom.appearance.feet_bone_group)
                                .unwrap_or_default(),
                            hands_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data.custom_skater.custom.appearance.hands_bone_group,
                                )
                                .unwrap_or_default(),
                            head_bone_group: flags
                                .scales
                                .then_some(cas.data.custom_skater.custom.appearance.head_bone_group)
                                .unwrap_or_default(),
                            headtop_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data.custom_skater.custom.appearance.headtop_bone_group,
                                )
                                .unwrap_or_default(),
                            jaw_bone_group: flags
                                .scales
                                .then_some(cas.data.custom_skater.custom.appearance.jaw_bone_group)
                                .unwrap_or_default(),
                            lower_arm_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data
                                        .custom_skater
                                        .custom
                                        .appearance
                                        .lower_arm_bone_group,
                                )
                                .unwrap_or_default(),
                            lower_leg_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data
                                        .custom_skater
                                        .custom
                                        .appearance
                                        .lower_leg_bone_group,
                                )
                                .unwrap_or_default(),
                            nose_bone_group: flags
                                .scales
                                .then_some(cas.data.custom_skater.custom.appearance.nose_bone_group)
                                .unwrap_or_default(),
                            object_scaling: flags
                                .scales
                                .then_some(cas.data.custom_skater.custom.appearance.object_scaling)
                                .unwrap_or_default(),
                            stomach_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data.custom_skater.custom.appearance.stomach_bone_group,
                                )
                                .unwrap_or_default(),
                            torso_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data.custom_skater.custom.appearance.torso_bone_group,
                                )
                                .unwrap_or_default(),
                            upper_arm_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data
                                        .custom_skater
                                        .custom
                                        .appearance
                                        .upper_arm_bone_group,
                                )
                                .unwrap_or_default(),
                            upper_leg_bone_group: flags
                                .scales
                                .then_some(
                                    cas.data
                                        .custom_skater
                                        .custom
                                        .appearance
                                        .upper_leg_bone_group,
                                )
                                .unwrap_or_default(),
                            ..Default::default()
                        },
                    },
                },
                story_skater: cas::StorySkater {
                    tricks: flags
                        .trickset
                        .then_some(cas.data.story_skater.tricks)
                        .unwrap_or_default(),
                },
            },
        }
    }
}
