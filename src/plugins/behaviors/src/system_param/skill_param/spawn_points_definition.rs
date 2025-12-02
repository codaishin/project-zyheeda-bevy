use crate::{
	components::fix_points::FixPointsDefinition,
	system_param::skill_param::SpawnPointContextMut,
};
use common::traits::{
	handles_animations::BoneName,
	handles_skill_behaviors::SkillSpawner,
	handles_skills_control::SpawnPointsDefinition,
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
		system_param::skill_param::SkillParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContextMut, handles_skills_control::SkillSpawnPoints},
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
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillSpawnPoints { entity }).unwrap();

				ctx.insert_spawn_point_definition(map_clone.clone());
			})?;

		assert_eq!(
			Some(&FixPointsDefinition(map)),
			app.world().entity(entity).get::<FixPointsDefinition>(),
		);
		Ok(())
	}
}
