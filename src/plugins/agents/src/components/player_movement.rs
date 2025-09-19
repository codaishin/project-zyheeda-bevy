use bevy::prelude::Component;
use common::{
	tools::{
		Units,
		UnitsPerSecond,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::GetProperty,
		animation::{Animation, AnimationAsset, PlayMode},
	},
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

impl GetProperty<Speed> for PlayerMovement {
	fn get_property(&self) -> UnitsPerSecond {
		match self.mode {
			MovementMode::Fast => self.fast.speed.0,
			MovementMode::Slow => self.slow.speed.0,
		}
	}
}

impl GetProperty<ColliderRadius> for PlayerMovement {
	fn get_property(&self) -> Units {
		self.collider_radius.0
	}
}

impl GetProperty<Option<MovementAnimation>> for PlayerMovement {
	fn get_property(&self) -> Option<&Animation> {
		match self.mode {
			MovementMode::Fast => Some(&self.fast.animation.0),
			MovementMode::Slow => Some(&self.slow.animation.0),
		}
	}
}
