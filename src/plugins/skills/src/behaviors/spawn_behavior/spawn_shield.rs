use super::OnSkillStop;
use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use behaviors::components::shield::Shield;
use bevy::prelude::{BuildChildren, Commands, Entity, SpatialBundle, Transform};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield {
	stoppable: bool,
}

impl SpawnShield {
	pub fn apply(
		&self,
		commands: &mut Commands,
		_: &SkillCaster,
		spawner: &SkillSpawner,
		_: &Target,
	) -> (Entity, Entity, OnSkillStop) {
		let SkillSpawner(entity, transform) = spawner;

		let contact = commands
			.spawn((
				Shield { location: *entity },
				SpatialBundle::from_transform(Transform::from(*transform)),
			))
			.id();
		let projection = commands.spawn_empty().set_parent(contact).id();

		if self.stoppable {
			(contact, projection, OnSkillStop::Stop(contact))
		} else {
			(contact, projection, OnSkillStop::Ignore)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::App,
		ecs::system::RunSystemOnce,
		prelude::{Entity, SpatialBundle, Transform},
	};
	use common::assert_bundle;

	fn shield(
		sh: SpawnShield,
		caster: SkillCaster,
		spawn: SkillSpawner,
		target: Target,
	) -> impl Fn(Commands) -> (Entity, Entity, OnSkillStop) {
		move |mut commands| sh.apply(&mut commands, &caster, &spawn, &target)
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_spacial_bundle() {
		let mut app = setup();
		let spawner_transform = Transform::from_xyz(1., 2., 3.);

		let (entity, ..) = app.world_mut().run_system_once(shield(
			SpawnShield { stoppable: true },
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)).with_transform(spawner_transform),
			Target::default(),
		));

		assert_bundle!(
			SpatialBundle,
			&app,
			app.world().entity(entity),
			With::assert(|transform: &Transform| assert_eq!(transform, &spawner_transform))
		);
	}

	#[test]
	fn spawn_shield() {
		let mut app = setup();
		let spawner_entity = Entity::from_raw(43);

		let (entity, ..) = app.world_mut().run_system_once(shield(
			SpawnShield { stoppable: true },
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(spawner_entity),
			Target::default(),
		));
		assert_eq!(
			Some(&Shield {
				location: spawner_entity
			}),
			app.world().entity(entity).get::<Shield>()
		);
	}

	#[test]
	fn spawn_stoppable() {
		let mut app = setup();

		let (entity, projection, on_skill_stop) = app.world_mut().run_system_once(shield(
			SpawnShield { stoppable: true },
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::default(),
		));

		assert_eq!(OnSkillStop::Stop(entity), on_skill_stop)
	}

	#[test]
	fn spawn_non_stoppable() {
		let mut app = setup();

		let (.., on_skill_stop) = app.world_mut().run_system_once(shield(
			SpawnShield { stoppable: false },
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::default(),
		));

		assert_eq!(OnSkillStop::Ignore, on_skill_stop)
	}
}
