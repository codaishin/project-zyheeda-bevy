use bevy::prelude::Component;
use common::{
	tools::{Units, UnitsPerSecond, speed::Speed},
	traits::{accessors::get::View, handles_movement::RequiredClearance},
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

impl View<Speed> for MovementConfig {
	fn view(&self) -> UnitsPerSecond {
		self.speed
	}
}

impl View<RequiredClearance> for MovementConfig {
	fn view(&self) -> Units {
		self.collider_radius
	}
}
