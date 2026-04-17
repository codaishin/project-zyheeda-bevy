use bevy::prelude::*;
use common::{
	tools::UnitsPerSecond,
	traits::handles_movement::{MovementSpeed, RequiredClearance, SpeedToggle},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(SpeedIndex)]
pub(crate) struct Config {
	pub(crate) speed: MovementSpeed,
	pub(crate) required_clearance: RequiredClearance,
}

impl Index<SpeedIndex> for Config {
	type Output = UnitsPerSecond;

	fn index(&self, SpeedIndex(toggle): SpeedIndex) -> &Self::Output {
		let [left, right] = match &self.speed {
			MovementSpeed::Fixed(speed) => return speed,
			MovementSpeed::Variable(variable) => variable,
		};

		match toggle {
			SpeedToggle::Left => left,
			SpeedToggle::Right => right,
		}
	}
}

#[derive(
	Component, SavableComponent, Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize,
)]
#[savable_component(id = "current movement speed")]
pub(crate) struct SpeedIndex(pub(crate) SpeedToggle);
