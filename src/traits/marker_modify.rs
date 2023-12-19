pub mod track;

use crate::errors::Error;
use bevy::ecs::system::EntityCommands;

pub trait MarkerModify {
	fn insert_markers(&self, agent: &mut EntityCommands) -> Result<(), Error>;
	fn remove_markers(&self, agent: &mut EntityCommands) -> Result<(), Error>;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use bevy::{
		ecs::{component::Component, system::Commands},
		prelude::Entity,
	};

	#[derive(Component)]
	pub struct FakeResult {
		pub result: Result<(), Error>,
	}

	pub fn insert_system<TModify: MarkerModify>(
		agent: Entity,
		modify: TModify,
	) -> impl Fn(Commands) {
		move |mut commands| {
			let mut agent = commands.entity(agent);
			let result = modify.insert_markers(&mut agent);
			agent.insert(FakeResult { result });
		}
	}

	pub fn remove_system<TModify: MarkerModify>(
		agent: Entity,
		modify: TModify,
	) -> impl Fn(Commands) {
		move |mut commands| {
			let mut agent = commands.entity(agent);
			let result = modify.remove_markers(&mut agent);
			agent.insert(FakeResult { result });
		}
	}
}
