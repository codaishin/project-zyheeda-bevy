pub(crate) mod has_filter;

use bevy::{
	ecs::{
		query::{QueryData, QueryFilter, QueryItem},
		system::EntityCommands,
	},
	prelude::*,
};
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

pub(crate) trait MovementUpdate {
	type TComponents<'a>: QueryData;
	type TConstraint: QueryFilter;

	fn update(
		&self,
		agent: &mut EntityCommands,
		components: QueryItem<Self::TComponents<'_>>,
		speed: UnitsPerSecond,
	) -> IsDone;
}
