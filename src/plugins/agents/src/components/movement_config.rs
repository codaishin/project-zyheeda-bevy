use bevy::prelude::Component;
use common::{
	tools::{Units, UnitsPerSecond, speed::Speed},
	traits::handles_movement::RequiredClearance,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement config")]
pub struct MovementConfig {
	pub(crate) collider_radius: Units,
	pub(crate) speed: UnitsPerSecond,
}

impl Default for MovementConfig {
	fn default() -> Self {
		Self {
			collider_radius: Units::from(0.5),
			speed: UnitsPerSecond::from(1.0),
		}
	}
}

impl From<&'_ MovementConfig> for Speed {
	fn from(config: &MovementConfig) -> Self {
		Speed(config.speed)
	}
}

impl From<&'_ MovementConfig> for RequiredClearance {
	fn from(config: &'_ MovementConfig) -> Self {
		RequiredClearance(config.collider_radius)
	}
}
