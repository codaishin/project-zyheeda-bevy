pub(crate) mod bundle;
pub(crate) mod has_filter;
pub(crate) mod movement;
pub(crate) mod movement_config;

use crate::components::MovementMode;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::animation::Animation,
};

pub type Vec2Radians = Vec2;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct IsDone(bool);

impl IsDone {
	pub fn is_done(&self) -> bool {
		self.0
	}
}

impl From<bool> for IsDone {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

pub(crate) trait MovementData {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode);
}

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}

pub(crate) trait MoveTogether {
	fn entity(&self) -> Option<Entity>;
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}

pub(crate) trait MovementPositionBased {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone;
}

pub(crate) trait MovementVelocityBased {
	fn update(&self, agent: &mut EntityCommands, position: Vec3, speed: UnitsPerSecond) -> IsDone;
}

pub(crate) trait Cleanup {
	fn cleanup(&self, agent: &mut EntityCommands);
}

pub trait RemoveComponent<T: Bundle> {
	fn get_remover() -> fn(&mut EntityCommands);
}

pub(crate) trait GetAnimation {
	fn animation<'s>(&'s self, key: &MovementMode) -> &'s Animation;
}
