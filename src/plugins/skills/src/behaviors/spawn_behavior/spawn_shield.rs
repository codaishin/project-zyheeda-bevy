use super::OnSkillStop;
use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use behaviors::components::ForceShield;
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, SpatialBundle, Transform},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield {
	stoppable: bool,
	shield_type: ShieldType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ShieldType {
	Force,
}

impl SpawnShield {
	pub fn apply<'a>(
		&self,
		commands: &'a mut Commands,
		_: &SkillCaster,
		spawner: &SkillSpawner,
		_: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		let SkillSpawner(entity, transform) = spawner;

		let shield = match self.shield_type {
			ShieldType::Force => ForceShield { location: *entity },
		};

		let entity = commands.spawn((
			shield,
			SpatialBundle::from_transform(Transform::from(*transform)),
		));

		if self.stoppable {
			let id = entity.id();
			(entity, OnSkillStop::Stop(id))
		} else {
			(entity, OnSkillStop::Ignore)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use behaviors::components::ForceShield;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		prelude::{Entity, SpatialBundle, Transform},
	};
	use common::{assert_bundle, test_tools::utils::SingleThreadedApp};

	fn shield(
		sh: SpawnShield,
		caster: SkillCaster,
		spawn: SkillSpawner,
		target: Target,
	) -> impl Fn(Commands) -> (Entity, OnSkillStop) {
		move |mut commands| {
			let (entity, on_skill_stop) = sh.apply(&mut commands, &caster, &spawn, &target);
			(entity.id(), on_skill_stop)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_spacial_bundle() {
		let mut app = setup();
		let spawner_transform = Transform::from_xyz(1., 2., 3.);

		let (entity, ..) = app.world_mut().run_system_once(shield(
			SpawnShield {
				stoppable: true,
				shield_type: ShieldType::Force,
			},
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
	fn spawn_force_shield() {
		let mut app = setup();
		let spawner_entity = Entity::from_raw(43);

		let (entity, ..) = app.world_mut().run_system_once(shield(
			SpawnShield {
				stoppable: true,
				shield_type: ShieldType::Force,
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(spawner_entity),
			Target::default(),
		));
		assert_eq!(
			Some(&ForceShield {
				location: spawner_entity
			}),
			app.world().entity(entity).get::<ForceShield>()
		);
	}

	#[test]
	fn spawn_stoppable() {
		let mut app = setup();

		let (entity, on_skill_stop) = app.world_mut().run_system_once(shield(
			SpawnShield {
				stoppable: true,
				shield_type: ShieldType::Force,
			},
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
			SpawnShield {
				stoppable: false,
				shield_type: ShieldType::Force,
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::default(),
		));

		assert_eq!(OnSkillStop::Ignore, on_skill_stop)
	}
}
