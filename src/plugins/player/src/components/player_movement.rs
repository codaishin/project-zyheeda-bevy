use bevy::prelude::Component;
use common::{
	tools::{
		UnitsPerSecond,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::animation::{Animation, AnimationAsset, PlayMode},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PlayerMovement {
	pub(crate) collider_radius: ColliderRadius,
	pub(crate) mode: MovementMode,
	pub(crate) fast: Config,
	pub(crate) slow: Config,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub(crate) enum MovementMode {
	#[default]
	Fast,
	Slow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Config {
	pub(crate) speed: Speed,
	pub(crate) animation: MovementAnimation,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			speed: UnitsPerSecond::default().into(),
			animation: Animation::new(AnimationAsset::from(""), PlayMode::Replay).into(),
		}
	}
}

impl From<&PlayerMovement> for Speed {
	fn from(
		PlayerMovement {
			mode, fast, slow, ..
		}: &PlayerMovement,
	) -> Self {
		match mode {
			MovementMode::Fast => fast.speed,
			MovementMode::Slow => slow.speed,
		}
	}
}

impl From<&PlayerMovement> for ColliderRadius {
	fn from(
		PlayerMovement {
			collider_radius, ..
		}: &PlayerMovement,
	) -> Self {
		*collider_radius
	}
}

impl<'a> From<&'a PlayerMovement> for Option<&'a MovementAnimation> {
	fn from(
		PlayerMovement {
			mode, fast, slow, ..
		}: &'a PlayerMovement,
	) -> Self {
		match mode {
			MovementMode::Fast => Some(&fast.animation),
			MovementMode::Slow => Some(&slow.animation),
		}
	}
}
