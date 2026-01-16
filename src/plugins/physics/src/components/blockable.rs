use crate::components::{interaction_target::InteractionTarget, skill_transform::SkillTransforms};
use bevy::prelude::*;
use common::traits::handles_physics::PhysicalObject;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(InteractionTarget, GlobalTransform, SkillTransforms)]
pub struct Blockable(pub(crate) PhysicalObject);

impl From<PhysicalObject> for Blockable {
	fn from(interaction: PhysicalObject) -> Self {
		Self(interaction)
	}
}
