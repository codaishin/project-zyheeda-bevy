use crate::{
	components::fix_points::FixPointsDefinition,
	system_param::skill_param::SpawnPointContextMut,
};
use common::{
	tools::bone::Bone,
	traits::{
		handles_skill_behaviors::SkillSpawner,
		handles_skills_control::SpawnPointsDefinition,
	},
};
use std::collections::HashMap;

impl SpawnPointsDefinition for SpawnPointContextMut<'_> {
	fn insert_spawn_point_definition(&mut self, definition: HashMap<Bone<'static>, SkillSpawner>) {
		self.entity.try_insert(FixPointsDefinition(definition));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::fix_points::FixPointsDefinition,
		system_param::skill_param::SkillParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::EntityContextMut, handles_skills_control::SkillSpawnPoints},
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
			(Bone("a"), SkillSpawner::Neutral),
			(Bone("b"), SkillSpawner::Slot(SlotKey(42))),
		]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_entity_context_mut(&mut p, entity, SkillSpawnPoints)
						.unwrap();

				ctx.insert_spawn_point_definition(map_clone.clone());
			})?;

		assert_eq!(
			Some(&FixPointsDefinition(map)),
			app.world().entity(entity).get::<FixPointsDefinition>(),
		);
		Ok(())
	}
}
