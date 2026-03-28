use bevy::prelude::*;
use common::{
	tools::{Units, UnitsPerSecond},
	traits::handles_movement::MovementSpeed,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(SpeedIndex)]
pub(crate) struct Config {
	pub(crate) speed: MovementSpeed,
	pub(crate) required_clearance: Units,
	pub(crate) ground_offset: Vec3,
}

impl Index<SpeedIndex> for Config {
	type Output = UnitsPerSecond;

	fn index(&self, index: SpeedIndex) -> &Self::Output {
		let [default, toggled] = match &self.speed {
			MovementSpeed::Fixed(speed) => return speed,
			MovementSpeed::Variable(variable) => variable,
		};

		match index {
			SpeedIndex::Default => default,
			SpeedIndex::Toggled => toggled,
		}
	}
}

#[derive(
	Component, SavableComponent, Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize,
)]
#[savable_component(id = "current movement speed")]
pub(crate) enum SpeedIndex {
	#[default]
	Default,
	Toggled,
}
