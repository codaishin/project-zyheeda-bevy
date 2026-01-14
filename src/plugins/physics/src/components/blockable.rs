use crate::components::interaction_target::InteractionTarget;
use bevy::prelude::*;
use common::traits::handles_physics::PhysicalObject;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(InteractionTarget)]
pub struct Blockable(pub(crate) PhysicalObject);

impl From<PhysicalObject> for Blockable {
	fn from(interaction: PhysicalObject) -> Self {
		Self(interaction)
	}
}
