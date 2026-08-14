use crate::components::skill_transform::SkillTransforms;
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(GlobalTransform, SkillTransforms)]
pub struct Blockable(pub(crate) PhysicalObject);

impl From<PhysicalObject> for Blockable {
	fn from(interaction: PhysicalObject) -> Self {
		Self(interaction)
	}
}
