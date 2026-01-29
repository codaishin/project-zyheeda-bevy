use crate::{
	components::skill::{CreatedFrom, Skill},
	system_params::skill_spawner::SkillSpawnerMut,
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Spawn, SpawnArgs},
};

impl Spawn for SkillSpawnerMut<'_, '_> {
	fn spawn(
		&mut self,
		SpawnArgs {
			contact,
			projection,
			lifetime,
			contact_effects,
			projection_effects,
		}: SpawnArgs,
	) -> PersistentEntity {
		let skill = Skill {
			created_from: CreatedFrom::Spawn,
			lifetime,
			contact,
			contact_effects,
			projection,
			projection_effects,
		};
		let persistent_entity = PersistentEntity::default();
		self.commands.spawn((persistent_entity, skill));

		persistent_entity
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_params::skill_spawner::SkillSpawnerMut;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		CommonPlugin,
		tools::Units,
		traits::handles_skill_physics::{
			Contact,
			ContactShape,
			Motion,
			Projection,
			ProjectionShape,
			SkillCaster,
			SkillSpawner,
		},
	};
	use std::{collections::HashSet, sync::LazyLock};
	use testing::{SingleThreadedApp, assert_count};

	static CONTACT: LazyLock<Contact> = LazyLock::new(|| Contact {
		shape: ContactShape::Sphere {
			radius: Units::from(0.5),
			hollow_collider: false,
			destroyed_by: HashSet::from([]),
		},
		motion: Motion::HeldBy {
			caster: SkillCaster(PersistentEntity::default()),
			spawner: SkillSpawner::Neutral,
		},
	});

	static PROJECTION: LazyLock<Projection> = LazyLock::new(|| Projection {
		shape: ProjectionShape::Sphere {
			radius: Units::from(1.),
		},
		offset: None,
	});

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);

		app
	}

	mod spawn {
		use super::*;

		#[test]
		fn persistent_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone()));
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
					p.spawn(SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone()));
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<&Skill, With<PersistentEntity>>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(
				&Skill {
					lifetime: None,
					created_from: CreatedFrom::Spawn,
					contact: CONTACT.clone(),
					contact_effects: vec![],
					projection: PROJECTION.clone(),
					projection_effects: vec![],
				},
				skill
			);
			Ok(())
		}
	}

	mod returned_skill {
		use super::*;
		use common::{effects::force::Force, traits::handles_skill_physics::Effect};
		use std::{fmt::Debug, time::Duration};

		#[derive(Component, Debug, PartialEq)]
		struct _Marker;

		#[test]
		fn get_root_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			let root = app
				.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone()))
				})?;

			let mut skills = app.world_mut().query::<&PersistentEntity>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(root, *skill);
			Ok(())
		}

		#[test]
		fn set_lifetime() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(
						SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone())
							.with_lifetime(Duration::from_millis(42)),
					);
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<&Skill, With<PersistentEntity>>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(Some(Duration::from_millis(42)), skill.lifetime);
			Ok(())
		}

		#[test]
		fn insert_on_contact() -> Result<(), RunSystemError> {
			let mut app = setup();
			let effect = Effect::Force(Force);

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(
						SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone())
							.with_contact_effects(vec![effect]),
					);
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<&Skill, With<PersistentEntity>>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(vec![effect], skill.contact_effects);
			Ok(())
		}

		#[test]
		fn insert_on_projection() -> Result<(), RunSystemError> {
			let mut app = setup();
			let effect = Effect::Force(Force);

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(
						SpawnArgs::with_shape(CONTACT.clone(), PROJECTION.clone())
							.with_projection_effects(vec![effect]),
					);
				})?;

			let mut skills = app
				.world_mut()
				.query_filtered::<&Skill, With<PersistentEntity>>();
			let [skill] = assert_count!(1, skills.iter(app.world()));
			assert_eq!(vec![effect], skill.projection_effects);
			Ok(())
		}
	}
}
