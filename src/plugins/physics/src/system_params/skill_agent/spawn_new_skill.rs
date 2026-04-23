use crate::{
	components::skill::{CreatedFrom, Skill},
	system_params::skill_agent::SkillAgentMut,
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Spawn, SpawnArgs},
};

impl Spawn for SkillAgentMut<'_, '_> {
	fn spawn(&mut self, args: SpawnArgs) -> PersistentEntity {
		let persistent_entity = PersistentEntity::default();

		self.commands.spawn((
			Skill {
				shape: args.shape.clone(),
				created_from: CreatedFrom::Spawn,
				contact_effects: args.contact_effects.to_vec(),
				projection_effects: args.projection_effects.to_vec(),
				caster: args.caster,
				mount: args.mount,
			},
			persistent_entity,
		));

		persistent_entity
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::{Spawn, *};
	use crate::system_params::skill_agent::SkillAgentMut;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		CommonPlugin,
		traits::handles_skill_physics::{SkillCaster, SkillMount, SkillShape, shield::Shield},
	};
	use std::sync::LazyLock;
	use testing::{SingleThreadedApp, assert_count};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);

		app
	}

	static ARGS: LazyLock<SpawnArgs> = LazyLock::new(|| SpawnArgs {
		shape: &SkillShape::Shield(Shield),
		contact_effects: &[],
		projection_effects: &[],
		caster: SkillCaster(PersistentEntity::default()),
		mount: SkillMount::Center,
	});

	mod spawn {
		use super::*;

		#[test]
		fn persistent_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					p.spawn(*ARGS);
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
				.run_system_once(move |mut p: SkillAgentMut| {
					p.spawn(*ARGS);
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
					mount: ARGS.mount,
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
				.run_system_once(move |mut p: SkillAgentMut| p.spawn(*ARGS))?;

			let mut skills = app.world_mut().query::<&PersistentEntity>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(root, *skill);
			Ok(())
		}
	}
}
