use crate::system_params::skill_spawner::SkillSpawnerMut;
use common::traits::{
	accessors::get::GetMut,
	handles_skill_physics::{Despawn, SkillEntity},
};

impl Despawn for SkillSpawnerMut<'_, '_> {
	fn despawn(&mut self, SkillEntity(entity): SkillEntity) {
		let Some(entity) = self.commands.get_mut(&entity) else {
			return;
		};

		if !self.skills.contains(entity.id()) {
			return;
		}

		entity.try_despawn();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::skill::Skill;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{CommonPlugin, components::persistent_entity::PersistentEntity};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);

		app
	}

	#[test]
	fn despawn_skill() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let entity = app.world_mut().spawn((Skill, persistent_entity)).id();

		app.world_mut()
			.run_system_once(move |mut p: SkillSpawnerMut| {
				p.despawn(SkillEntity(persistent_entity));
			})?;

		assert!(app.world().get_entity(entity).is_err());
		Ok(())
	}

	#[test]
	fn do_not_despawn_non_skill() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let entity = app.world_mut().spawn(persistent_entity).id();

		app.world_mut()
			.run_system_once(move |mut p: SkillSpawnerMut| {
				p.despawn(SkillEntity(persistent_entity));
			})?;

		assert!(app.world().get_entity(entity).is_ok());
		Ok(())
	}
}
