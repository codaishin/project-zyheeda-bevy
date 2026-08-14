use crate::system_params::skill_agent::SkillAgentMut;
use common::prelude::*;

impl DespawnSkill for SkillAgentMut<'_, '_> {
	fn despawn_skill(&mut self, SkillEntity(entity): SkillEntity) {
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
	use super::{DespawnSkill, *};
	use crate::components::skill::{CreatedFrom, Skill};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));

		app
	}

	fn skill() -> Skill {
		Skill {
			created_from: CreatedFrom::Save,
			shape: SkillShape::Shield(Shield),
			contact_effects: vec![],
			projection_effects: vec![],
			caster: SkillCaster(PersistentEntity::default()),
			mount: SkillMount::Center,
		}
	}

	#[test]
	fn despawn_skill() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let entity = app.world_mut().spawn((skill(), persistent_entity)).id();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				p.despawn_skill(SkillEntity(persistent_entity));
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
			.run_system_once(move |mut p: SkillAgentMut| {
				p.despawn_skill(SkillEntity(persistent_entity));
			})?;

		assert!(app.world().get_entity(entity).is_ok());
		Ok(())
	}
}
