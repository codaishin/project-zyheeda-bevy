use crate::{
	components::skill::{CreatedFrom, Skill},
	system_params::skill_spawner::SkillSpawnerMut,
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Spawn, SpawnArgs},
};

impl Spawn for SkillSpawnerMut<'_, '_> {
	fn spawn(&mut self, args: SpawnArgs) -> PersistentEntity {
		let persistent_entity = PersistentEntity::default();

		self.commands.spawn((
			Skill {
				shape: args.shape.clone(),
				created_from: CreatedFrom::Spawn,
				contact_effects: args.contact_effects.to_vec(),
				projection_effects: args.projection_effects.to_vec(),
				caster: args.caster,
				spawner: args.spawner,
				target: args.target,
			},
			persistent_entity,
		));

		persistent_entity
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_params::skill_spawner::SkillSpawnerMut;
	use bevy::{
		asset::uuid::uuid,
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		CommonPlugin,
		traits::handles_skill_physics::{
			SkillCaster,
			SkillShape,
			SkillSpawner,
			SkillTarget,
			shield::Shield,
		},
	};
	use testing::{SingleThreadedApp, assert_count};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);

		app
	}

	const ARGS: SpawnArgs = SpawnArgs {
		shape: &SkillShape::Shield(Shield),
		contact_effects: &[],
		projection_effects: &[],
		caster: SkillCaster(PersistentEntity::from_uuid(uuid!(
			"3db021df-666e-4858-8fc4-845d0639a2e7"
		))),
		spawner: SkillSpawner::Neutral,
		target: SkillTarget::Entity(PersistentEntity::from_uuid(uuid!(
			"ae8d9c8c-cc4b-4ea0-a20d-f63992a9173f"
		))),
	};

	mod spawn {
		use super::*;

		#[test]
		fn persistent_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(ARGS);
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<(), With<PersistentEntity>>();
			assert_count!(1, skills.iter(app.world()));
			Ok(())
		}

		#[test]
		fn skill() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(ARGS);
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<&Skill, With<PersistentEntity>>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(
				&Skill {
					created_from: CreatedFrom::Spawn,
					shape: ARGS.shape.clone(),
					contact_effects: ARGS.contact_effects.to_vec(),
					projection_effects: ARGS.projection_effects.to_vec(),
					caster: ARGS.caster,
					spawner: ARGS.spawner,
					target: ARGS.target
				},
				skill
			);
			Ok(())
		}
	}

	mod returned_skill {
		use super::*;

		#[test]
		fn get_root_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			let root = app
				.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| p.spawn(ARGS))?;

			let mut skills = app.world_mut().query::<&PersistentEntity>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(root, *skill);
			Ok(())
		}
	}
}
