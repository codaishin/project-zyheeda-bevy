use bevy::prelude::Component;
use common::{
	tools::{
		UnitsPerSecond,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::{Getter, GetterRefOptional},
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

impl Getter<Speed> for PlayerMovement {
	fn get(&self) -> Speed {
		match self.mode {
			MovementMode::Fast => self.fast.speed,
			MovementMode::Slow => self.slow.speed,
		}
	}
}

impl Getter<ColliderRadius> for PlayerMovement {
	fn get(&self) -> ColliderRadius {
		self.collider_radius
	}
}

impl GetterRefOptional<MovementAnimation> for PlayerMovement {
	fn get(&self) -> Option<&MovementAnimation> {
		match self.mode {
			MovementMode::Fast => Some(&self.fast.animation),
			MovementMode::Slow => Some(&self.slow.animation),
		}
	}
}
