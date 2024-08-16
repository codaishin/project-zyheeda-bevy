use behaviors::components::gravity_well::GravityWell;
use bevy::ecs::system::EntityCommands;
use serde::{Deserialize, Serialize};

use crate::behaviors::{SkillCaster, SkillSpawner, Target};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartGravity;

impl StartGravity {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) {
		entity.insert(GravityWell);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use behaviors::components::gravity_well::GravityWell;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		prelude::{Commands, Entity},
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn gravity(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartGravity.apply(
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
	fn spawn_gravity_well() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(gravity);

		assert_eq!(
			Some(&GravityWell),
			app.world().entity(entity).get::<GravityWell>()
		);
	}
}
