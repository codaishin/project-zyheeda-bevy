use crate::{
	components::{
		effect::{force::ForceEffect, gravity::GravityEffect, health_damage::HealthDamageEffect},
		skill::Skill,
		skill_prefabs::{skill_contact::SkillContact, skill_projection::SkillProjection},
	},
	system_params::skill_spawner::SkillSpawnerMut,
};
use bevy::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	traits::{
		accessors::get::TryApplyOn,
		handles_skill_physics::{
			Contact,
			Effect,
			Projection,
			Skill as SkillTrait,
			SkillEntities,
			SkillRoot,
			Spawn,
		},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl Spawn for SkillSpawnerMut<'_, '_> {
	type TSkill<'c>
		= SkillCommands<'c>
	where
		Self: 'c;

	fn spawn(&mut self, contact: Contact, projection: Projection) -> SkillCommands<'_> {
		let persistent_entity = PersistentEntity::default();
		let contact = self
			.commands
			.spawn((Skill, SkillContact::from(contact), persistent_entity))
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

		SkillCommands {
			commands: self.commands.reborrow(),
			entities,
		}
	}
}

pub struct SkillCommands<'c> {
	commands: ZyheedaCommands<'c, 'c>,
	entities: SkillEntities,
}

impl SkillTrait for SkillCommands<'_> {
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

	fn insert_on_contact(&mut self, effect: Effect) {
		let entity = &self.entities.contact;

		self.commands.try_apply_on(entity, try_insert(effect));
	}

	fn insert_on_projection(&mut self, effect: Effect) {
		let entity = &self.entities.projection;

		self.commands.try_apply_on(entity, try_insert(effect));
	}
}

fn try_insert(effect: Effect) -> impl Fn(ZyheedaEntityCommands) {
	move |mut e| {
		match effect {
			Effect::Force(force) => e.try_insert(ForceEffect(force)),
			Effect::Gravity(gravity) => e.try_insert(GravityEffect(gravity)),
			Effect::HealthDamage(health_damage) => e.try_insert(HealthDamageEffect(health_damage)),
		};
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
			assert_eq!(Some(&Skill), skill.get::<Skill>());
			Ok(())
		}

		#[test]
		fn contact() -> Result<(), RunSystemError> {
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
					p.spawn(CONTACT.clone(), PROJECTION.clone());
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
		use std::fmt::Debug;

		use super::*;
		use common::{
			effects::{EffectApplies, force::Force, gravity::Gravity, health_damage::HealthDamage},
			tools::UnitsPerSecond,
		};
		use test_case::test_case;

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
		fn insert_on_root() -> Result<(), RunSystemError> {
			let mut app = setup();

			app.world_mut()
				.run_system_once(move |mut p: SkillSpawnerMut| {
					let mut skill = p.spawn(CONTACT.clone(), PROJECTION.clone());
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

		static GRAVITY: LazyLock<Gravity> = LazyLock::new(|| Gravity {
			strength: UnitsPerSecond::from(11.),
		});
		static HEALTH_DAMAGE: LazyLock<HealthDamage> =
			LazyLock::new(|| HealthDamage(42., EffectApplies::Once));

		#[test_case(Effect::Force(Force), ForceEffect(Force); "force")]
		#[test_case(Effect::Gravity(*GRAVITY), GravityEffect(*GRAVITY); "gravity")]
		#[test_case(Effect::HealthDamage(*HEALTH_DAMAGE), HealthDamageEffect(*HEALTH_DAMAGE); "damage")]
		fn insert_on_contact<T>(effect: Effect, component: T) -> Result<(), RunSystemError>
		where
			T: Component + Debug + PartialEq,
		{
			let mut app = setup();

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
			assert_eq!(Some(&component), skill.get::<T>());
			Ok(())
		}

		#[test_case(Effect::Force(Force), ForceEffect(Force); "force")]
		#[test_case(Effect::Gravity(*GRAVITY), GravityEffect(*GRAVITY); "gravity")]
		#[test_case(Effect::HealthDamage(*HEALTH_DAMAGE), HealthDamageEffect(*HEALTH_DAMAGE); "damage")]
		fn insert_on_projection<T>(effect: Effect, component: T) -> Result<(), RunSystemError>
		where
			T: Component + Debug + PartialEq,
		{
			let mut app = setup();

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
			let [projection] = assert_count!(1, get_children!(app, skill.id()));
			assert_eq!(Some(&component), projection.get::<T>());
			Ok(())
		}
	}
}
