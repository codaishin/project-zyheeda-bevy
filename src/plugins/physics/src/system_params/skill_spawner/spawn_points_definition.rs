use crate::{
	components::fix_points::FixPointsDefinition,
	system_params::skill_spawner::SpawnPointContextMut,
};
use common::{
	tools::bone_name::BoneName,
	traits::{
		handles_skill_behaviors::SkillSpawner,
		handles_skill_spawning::SpawnPointsDefinition,
	},
};
use std::collections::HashMap;

impl SpawnPointsDefinition for SpawnPointContextMut<'_> {
	fn insert_spawn_point_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>) {
		self.entity.try_insert(FixPointsDefinition(definition));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::fix_points::FixPointsDefinition,
		system_params::skill_spawner::SkillSpawnerMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContextMut, handles_skill_spawning::SkillSpawnPoints},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_fix_points_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let map = HashMap::from([
			(BoneName::from("a"), SkillSpawner::Neutral),
			(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
		]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillSpawnerMut| {
				let mut ctx =
					SkillSpawnerMut::get_context_mut(&mut p, SkillSpawnPoints { entity }).unwrap();

				ctx.insert_spawn_point_definition(map_clone.clone());
			})?;

		assert_eq!(
			Some(&FixPointsDefinition(map)),
			app.world().entity(entity).get::<FixPointsDefinition>(),
		);
		Ok(())
	}
}
