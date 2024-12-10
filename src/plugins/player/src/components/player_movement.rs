use bevy::prelude::Component;
use common::traits::{
	accessors::get::{GetterMut, GetterRef},
	animation::Animation,
	handles_behaviors::{MovementMode, Speed},
};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct PlayerMovement {
	pub(crate) mode: MovementMode,
	pub(crate) speeds: Config<Speed>,
	pub(crate) animations: Config<Animation>,
}

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct Config<T> {
	pub(crate) slow: T,
	pub(crate) fast: T,
}

impl GetterRef<Speed> for PlayerMovement {
	fn get(&self) -> &Speed {
		match self.mode {
			MovementMode::Slow => &self.speeds.slow,
			MovementMode::Fast => &self.speeds.fast,
		}
	}
}

impl GetterRef<Animation> for PlayerMovement {
	fn get(&self) -> &Animation {
		match self.mode {
			MovementMode::Slow => &self.animations.slow,
			MovementMode::Fast => &self.animations.fast,
		}
	}
}

impl GetterRef<MovementMode> for PlayerMovement {
	fn get(&self) -> &MovementMode {
		&self.mode
	}
}

impl GetterMut<MovementMode> for PlayerMovement {
	fn get_mut(&mut self) -> &mut MovementMode {
		&mut self.mode
	}
}
