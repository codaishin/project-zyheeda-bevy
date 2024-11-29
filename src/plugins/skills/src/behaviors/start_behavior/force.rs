use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use common::blocker::Blocker;
use interactions::components::force::Force;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartForce;

impl StartForce {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) {
		entity.try_insert((Blocker::insert([Blocker::Force]), Force));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		prelude::{Commands, Entity},
	};
	use common::{blocker::BlockerInsertCommand, test_tools::utils::SingleThreadedApp};

	fn force(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartForce.apply(
			&mut entity,
			&SkillCaster::from(Entity::from_raw(42)),
			&SkillSpawner::from(Entity::from_raw(43)),
			&Target::default(),
		);
		entity.id()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_force() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(force);

		assert_eq!(
			Some(&Blocker::insert([Blocker::Force])),
			app.world().entity(entity).get::<BlockerInsertCommand>()
		);
	}

	#[test]
	fn spawn_force_marker() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(force);

		assert_eq!(Some(&Force), app.world().entity(entity).get::<Force>());
	}
}
