use bevy::prelude::*;
use common::traits::handles_skill_physics::SkillTarget;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Target(pub(crate) Option<SkillTarget>);
