use bevy::prelude::Component;
use common::{
	tools::{Units, UnitsPerSecond},
	traits::animation::Animation,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct MovementConfig {
	pub(crate) collider_radius: Units,
	pub(crate) speed: UnitsPerSecond,
	pub(crate) animation: Option<Animation>,
}
