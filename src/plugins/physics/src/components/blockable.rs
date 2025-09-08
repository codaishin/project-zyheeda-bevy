use bevy::prelude::*;
use common::traits::handles_physics::PhysicalObject;

#[derive(Component, Debug, PartialEq)]
pub struct Blockable(pub(crate) PhysicalObject);

impl From<PhysicalObject> for Blockable {
	fn from(interaction: PhysicalObject) -> Self {
		Self(interaction)
	}
}
