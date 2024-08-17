use super::OnSkillStop;
use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use behaviors::components::projectile::{sub_type::SubType, Projectile};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, SpatialBundle, Transform},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile {
	stoppable: bool,
	sub_type: SubType,
}

impl SpawnProjectile {
	pub fn apply<'a>(
		&self,
		commands: &'a mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		_: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		let SkillCaster(.., caster) = caster;
		let SkillSpawner(.., spawner) = spawner;

		let entity = commands.spawn((
			Projectile {
				direction: caster.forward(),
				range: 10.,
				sub_type: self.sub_type,
			},
			SpatialBundle::from_transform(Transform::from(*spawner)),
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
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		math::{Dir3, Vec3},
		prelude::{Entity, SpatialBundle, Transform},
	};
	use common::{assert_bundle, assert_eq_approx, test_tools::utils::SingleThreadedApp};

	fn projectile(
		pr: SpawnProjectile,
		caster: SkillCaster,
		spawn: SkillSpawner,
		target: Target,
	) -> impl Fn(Commands) -> (Entity, OnSkillStop) {
		move |mut commands| {
			let (entity, on_skill_stop) = pr.apply(&mut commands, &caster, &spawn, &target);
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

		let (entity, ..) = app.world_mut().run_system_once(projectile(
			SpawnProjectile {
				stoppable: true,
				sub_type: SubType::Plasma,
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
	fn spawn_projectile() {
		let mut app = setup();
		let caster_forward = Dir3::try_from(Vec3::new(1., 0., 1.)).unwrap();
		let caster_transform = Transform::from_xyz(1., 2., 3.).looking_to(caster_forward, Vec3::Y);

		let (entity, ..) = app.world_mut().run_system_once(projectile(
			SpawnProjectile {
				stoppable: true,
				sub_type: SubType::Plasma,
			},
			SkillCaster::from(Entity::from_raw(42)).with_transform(caster_transform),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::default(),
		));

		let projectile = app.world().entity(entity).get::<Projectile>().unwrap();

		assert_eq_approx!(
			Projectile {
				direction: caster_forward,
				range: 10.,
				sub_type: SubType::Plasma
			},
			projectile,
			0.001
		);
	}

	#[test]
	fn spawn_stoppable() {
		let mut app = setup();

		let (entity, on_skill_stop) = app.world_mut().run_system_once(projectile(
			SpawnProjectile {
				stoppable: true,
				sub_type: SubType::Plasma,
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

		let (.., on_skill_stop) = app.world_mut().run_system_once(projectile(
			SpawnProjectile {
				stoppable: false,
				sub_type: SubType::Plasma,
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::default(),
		));

		assert_eq!(OnSkillStop::Ignore, on_skill_stop)
	}
}
