use super::OnSkillStop;
use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use behaviors::components::ground_targeted_aoe::GroundTargetedAoe;
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Transform},
};
use common::tools::Units;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe {
	pub max_range: Units,
	pub radius: Units,
	pub stoppable: bool,
}

impl SpawnGroundTargetedAoe {
	pub fn apply<'a>(
		&self,
		commands: &'a mut Commands,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		let SkillCaster(.., caster_transform) = caster;
		let Target { ray, .. } = target;

		let entity = commands.spawn(GroundTargetedAoe {
			caster: Transform::from(*caster_transform),
			target_ray: *ray,
			max_range: self.max_range,
			radius: self.radius,
		});

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
	use behaviors::components::ground_targeted_aoe::GroundTargetedAoe;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		math::{Ray3d, Vec3},
		prelude::{Entity, Transform},
		utils::default,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	fn ground_target(
		gt: SpawnGroundTargetedAoe,
		caster: SkillCaster,
		spawn: SkillSpawner,
		target: Target,
	) -> impl Fn(Commands) -> (Entity, OnSkillStop) {
		move |mut commands| {
			let (entity, on_skill_stop) = gt.apply(&mut commands, &caster, &spawn, &target);
			(entity.id(), on_skill_stop)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_ground_target() {
		let mut app = setup();
		let caster_transform = Transform::from_xyz(1., 2., 3.);
		let target_ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));

		let (entity, ..) = app.world_mut().run_system_once(ground_target(
			SpawnGroundTargetedAoe {
				max_range: Units::new(20.),
				radius: Units::new(8.),
				..default()
			},
			SkillCaster::from(Entity::from_raw(42)).with_transform(caster_transform),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::from(target_ray),
		));

		assert_eq!(
			Some(&GroundTargetedAoe {
				caster: caster_transform,
				target_ray,
				max_range: Units::new(20.),
				radius: Units::new(8.),
			}),
			app.world().entity(entity).get::<GroundTargetedAoe>()
		)
	}

	#[test]
	fn spawn_as_stoppable() {
		let mut app = setup();

		let (entity, on_skill_stop) = app.world_mut().run_system_once(ground_target(
			SpawnGroundTargetedAoe {
				stoppable: true,
				..default()
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::from(GroundTargetedAoe::DEFAULT_TARGET_RAY),
		));

		assert_eq!(OnSkillStop::Stop(entity), on_skill_stop);
	}

	#[test]
	fn spawn_as_non_stoppable() {
		let mut app = setup();

		let (.., on_skill_stop) = app.world_mut().run_system_once(ground_target(
			SpawnGroundTargetedAoe {
				stoppable: false,
				..default()
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::from(GroundTargetedAoe::DEFAULT_TARGET_RAY),
		));

		assert_eq!(OnSkillStop::Ignore, on_skill_stop);
	}
}
