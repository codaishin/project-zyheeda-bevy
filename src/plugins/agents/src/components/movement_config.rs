use bevy::prelude::Component;
use common::{
	tools::{Units, UnitsPerSecond},
	traits::animation::Animation,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MovementConfig {
	pub(crate) collider_radius: Units,
	pub(crate) speed: UnitsPerSecond,
	pub(crate) animation: Option<Animation>,
}

impl Default for MovementConfig {
	fn default() -> Self {
		Self {
			collider_radius: Units::from(0.5),
			speed: UnitsPerSecond::from(1.0),
			animation: None,
		}
	}
}
