use crate::{
	components::skill_prefabs::{skill_contact::SkillContact, skill_projection::SkillProjection},
	system_params::skill_spawner::SpawnNewSkillContextMut,
};
use bevy::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	traits::{
		accessors::get::TryApplyOn,
		handles_skill_physics::{
			Contact,
			Projection,
			Skill as SkillTrait,
			SkillEntities,
			SkillRoot,
			Spawn,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Spawn for SpawnNewSkillContextMut<'_> {
	type TSkill<'c>
		= Skill<'c>
	where
		Self: 'c;

	fn spawn(&mut self, contact: Contact, projection: Projection) -> Skill<'_> {
		let persistent_entity = PersistentEntity::default();
		let contact = self
			.commands
			.spawn((SkillContact::from(contact), persistent_entity))
			.id();
		let projection = self
			.commands
			.spawn((
				SkillProjection::from(projection),
				ChildOfPersistent(persistent_entity),
			))
			.id();

		let entities = SkillEntities {
			root: SkillRoot {
				entity: contact,
				persistent_entity,
			},
			contact,
			projection,
		};

		Skill {
			commands: self.commands.reborrow(),
			entities,
		}
	}
}

pub struct Skill<'c> {
	commands: ZyheedaCommands<'c, 'c>,
	entities: SkillEntities,
}

impl SkillTrait for Skill<'_> {
	fn root(&self) -> PersistentEntity {
		self.entities.root.persistent_entity
	}

	fn insert_on_root<T>(&mut self, bundle: T)
	where
		T: Bundle,
	{
		let entity = &self.entities.root.entity;

		self.commands.try_apply_on(entity, |mut e| {
			e.try_insert(bundle);
		});
	}

	fn insert_on_contact<T>(&mut self, bundle: T)
	where
		T: Bundle,
	{
		let entity = &self.entities.contact;

		self.commands.try_apply_on(entity, |mut e| {
			e.try_insert(bundle);
		});
	}

	fn insert_on_projection<T>(&mut self, bundle: T)
	where
		T: Bundle,
	{
		let entity = &self.entities.projection;

		self.commands.try_apply_on(entity, |mut e| {
			e.try_insert(bundle);
		});
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_params::skill_spawner::SkillSpawnerMut;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		CommonPlugin,
		tools::Units,
		traits::{
			accessors::get::GetContextMut,
			handles_skill_physics::{
				ContactShape,
				Motion,
				NewSkill,
				ProjectionShape,
				SkillCaster,
				SkillSpawner,
			},
		},
	};
	use std::{collections::HashSet, sync::LazyLock};
	use testing::{SingleThreadedApp, assert_count, get_children};

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
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					ctx.spawn(CONTACT.clone(), PROJECTION.clone());
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			assert_count!(1, skills);
			Ok(())
		}

		#[test]
		fn contact() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					ctx.spawn(CONTACT.clone(), PROJECTION.clone());
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(
				Some(&SkillContact::from(CONTACT.clone())),
				skill.get::<SkillContact>()
			);
			Ok(())
		}

		#[test]
		fn projection() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					ctx.spawn(CONTACT.clone(), PROJECTION.clone());
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			let [projection] = assert_count!(1, get_children!(app, skill.id()));
			assert_eq!(
				Some(&SkillProjection::from(PROJECTION.clone())),
				projection.get::<SkillProjection>()
			);
			Ok(())
		}
	}

	mod returned_skill {
		use super::*;

		#[derive(Component, Debug, PartialEq)]
		struct _Marker;

		#[test]
		fn get_root_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			let root = app
				.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					let skill = ctx.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.root()
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter_map(|e| e.get::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(root, *skill);
			Ok(())
		}

		#[test]
		fn insert_on_root() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					let mut skill = ctx.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.insert_on_root(_Marker);
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(Some(&_Marker), skill.get::<_Marker>());
			Ok(())
		}

		#[test]
		fn insert_on_contact() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					let mut skill = ctx.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.insert_on_contact(_Marker);
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(Some(&_Marker), skill.get::<_Marker>());
			Ok(())
		}

		#[test]
		fn insert_on_projection() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut ctx = SkillSpawnerMut::get_context_mut(&mut p, NewSkill).unwrap();
					let mut skill = ctx.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.insert_on_projection(_Marker);
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			let [projection] = assert_count!(1, get_children!(app, skill.id()));
			assert_eq!(Some(&_Marker), projection.get::<_Marker>());
			Ok(())
		}
	}
}
