pub mod skill;

use crate::behaviors::meta::Spawner;
use bevy::{ecs::system::EntityCommands, transform::components::Transform};

pub trait BehaviorExecution {
	fn run(&self, agent: &mut EntityCommands, spawner: &Spawner);
	fn stop(&self, agent: &mut EntityCommands);
	fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner);
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use bevy::{
		ecs::{
			component::Component,
			system::{Commands, Query},
		},
		prelude::Entity,
	};

	pub fn run_system<TExecute: BehaviorExecution + Component>(
		agent: Entity,
		spawner: Spawner,
	) -> impl FnMut(Commands, Query<&mut TExecute>) {
		move |mut commands, mut executes| {
			let execute = executes.single_mut();
			execute.run(&mut commands.entity(agent), &spawner);
		}
	}

	pub fn stop_system<TExecute: BehaviorExecution + Component>(
		agent: Entity,
	) -> impl FnMut(Commands, Query<&TExecute>) {
		move |mut commands, executes| {
			let execute = executes.single();
			execute.stop(&mut commands.entity(agent));
		}
	}
}
