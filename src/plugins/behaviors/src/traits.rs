pub(crate) mod bundle;
pub(crate) mod has_filter;

use bevy::{ecs::system::EntityCommands, prelude::*};
use common::tools::UnitsPerSecond;

pub type Vec2Radians = Vec2;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct IsDone(pub(crate) bool);

impl From<bool> for IsDone {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}

pub(crate) trait MoveTogether {
	fn entity(&self) -> Option<Entity>;
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
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
