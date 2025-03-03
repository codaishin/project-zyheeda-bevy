use bevy::prelude::Component;
use common::{
	tools::{UnitsPerSecond, movement_animation::MovementAnimation, speed::Speed},
	traits::{
		accessors::get::{Getter, GetterRefOptional},
		animation::{Animation, PlayMode},
		load_asset::Path,
	},
};

#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct PlayerMovement {
	pub(crate) mode: MovementMode,
	pub(crate) fast: Config,
	pub(crate) slow: Config,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) enum MovementMode {
	#[default]
	Fast,
	Slow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Config {
	pub(crate) speed: Speed,
	pub(crate) animation: MovementAnimation,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			speed: UnitsPerSecond::default().into(),
			animation: Animation::new(Path::from(""), PlayMode::Replay).into(),
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

impl GetterRefOptional<MovementAnimation> for PlayerMovement {
	fn get(&self) -> Option<&MovementAnimation> {
		match self.mode {
			MovementMode::Fast => Some(&self.fast.animation),
			MovementMode::Slow => Some(&self.slow.animation),
		}
	}
}
