use bevy::prelude::*;
use common::{tools::Units, traits::handles_movement::MovementSpeed};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(CurrentMovementSpeed)]
pub(crate) struct Config {
	pub(crate) speed: MovementSpeed,
	pub(crate) required_clearance: Units,
	pub(crate) ground_offset: Vec3,
}

#[derive(
	Component, SavableComponent, Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize,
)]
#[savable_component(id = "current movement speed")]
pub(crate) enum CurrentMovementSpeed {
	#[default]
	First,
	Second,
}
