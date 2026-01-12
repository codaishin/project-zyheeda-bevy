use crate::{
	components::skill::{CreatedFrom, Skill},
	system_params::skill_spawner::SkillSpawnerMut,
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Contact, Effect, Projection, Skill as SkillTrait, Spawn},
	zyheeda_commands::ZyheedaCommands,
};
use std::time::Duration;

impl Spawn for SkillSpawnerMut<'_, '_> {
	type TSkill<'c>
		= SkillCommands<'c>
	where
		Self: 'c;

	fn spawn(&mut self, contact: Contact, projection: Projection) -> SkillCommands<'_> {
		let skill = Skill {
			lifetime: None,
			created_from: CreatedFrom::Spawn,
			contact: contact.clone(),
			contact_effects: vec![],
			projection: projection.clone(),
			projection_effects: vec![],
		};
		let persistent_entity = PersistentEntity::default();

		SkillCommands {
			persistent_entity,
			commands: self.commands.reborrow(),
			skill,
		}
	}
}

pub struct SkillCommands<'c> {
	commands: ZyheedaCommands<'c, 'c>,
	persistent_entity: PersistentEntity,
	skill: Skill,
}

impl Drop for SkillCommands<'_> {
	fn drop(&mut self) {
		self.commands
			.spawn((self.persistent_entity, self.skill.clone()));
	}
}

impl SkillTrait for SkillCommands<'_> {
	fn root(&self) -> PersistentEntity {
		self.persistent_entity
	}

	fn set_lifetime(&mut self, duration: Duration) {
		self.skill.lifetime = Some(duration);
	}

	fn insert_on_contact(&mut self, effect: Effect) {
		self.skill.contact_effects.push(effect);
	}

	fn insert_on_projection(&mut self, effect: Effect) {
		self.skill.projection_effects.push(effect);
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
		traits::handles_skill_physics::{
			ContactShape,
			Motion,
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
					p.spawn(CONTACT.clone(), PROJECTION.clone());
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			assert_count!(1, skills);
			Ok(())
		}

		#[test]
		fn skill() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					p.spawn(CONTACT.clone(), PROJECTION.clone());
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(
				Some(&Skill {
					lifetime: None,
					created_from: CreatedFrom::Spawn,
					contact: CONTACT.clone(),
					contact_effects: vec![],
					projection: PROJECTION.clone(),
					projection_effects: vec![],
				}),
				skill.get::<Skill>()
			);
			Ok(())
		}
	}

	mod returned_skill {
		use super::*;
		use common::effects::force::Force;
		use std::fmt::Debug;

		#[derive(Component, Debug, PartialEq)]
		struct _Marker;

		#[test]
		fn get_root_entity() -> Result<(), RunSystemError> {
			let mut app = setup();

			let root = app
				.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let skill = p.spawn(CONTACT.clone(), PROJECTION.clone());
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
		fn set_lifetime() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut skill = p.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.set_lifetime(Duration::from_millis(42));
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(
				Some(Duration::from_millis(42)),
				skill.get::<Skill>().and_then(|s| s.lifetime)
			);
			Ok(())
		}

		#[test]
		fn insert_on_contact() -> Result<(), RunSystemError> {
			let mut app = setup();
			let effect = Effect::Force(Force);

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut skill = p.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.insert_on_contact(effect);
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(
				Some(&vec![effect]),
				skill.get::<Skill>().map(|s| &s.contact_effects)
			);
			Ok(())
		}

		#[test]
		fn insert_on_projection() -> Result<(), RunSystemError> {
			let mut app = setup();
			let effect = Effect::Force(Force);

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut skill = p.spawn(CONTACT.clone(), PROJECTION.clone());
					skill.insert_on_projection(effect);
				})?;

			let skills = app
				.world()
				.iter_entities()
				.filter(|e| e.contains::<PersistentEntity>());
			let [skill] = assert_count!(1, skills);
			assert_eq!(
				Some(&vec![effect]),
				skill.get::<Skill>().map(|s| &s.projection_effects)
			);
			Ok(())
		}
	}
}
