use crate::components::fix_points::FixPoints;
use bevy::prelude::*;
use common::traits::handles_skill_physics::SkillSpawner;

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = FixPoints)]
pub struct FixPointOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct FixPointSpawner(pub(crate) SkillSpawner);
