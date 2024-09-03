use super::OnSkillStop;
use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use behaviors::components::{
	ground_targeted_aoe::GroundTargetedAoe,
	Contact,
	LifeTime,
	Projection,
};
use bevy::{
	prelude::{BuildChildren, Commands, Entity, Transform},
	utils::default,
};
use common::{tools::Units, traits::try_insert_on::TryInsertOn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum LifeTimeData {
	#[default]
	UntilStopped,
	UntilOutlived(Duration),
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe {
	pub lifetime: LifeTimeData,
	pub max_range: Units,
	pub radius: Units,
}

impl SpawnGroundTargetedAoe {
	pub fn apply(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> (Entity, Entity, OnSkillStop) {
		let SkillCaster(.., caster_transform) = caster;
		let Target { ray, .. } = target;

		let contact = commands
			.spawn(GroundTargetedAoe::<Contact> {
				caster: Transform::from(*caster_transform),
				target_ray: *ray,
				max_range: self.max_range,
				radius: self.radius,
				..default()
			})
			.id();
		let projection = commands
			.spawn(GroundTargetedAoe::<Projection> {
				caster: Transform::from(*caster_transform),
				target_ray: *ray,
				max_range: self.max_range,
				radius: self.radius,
				..default()
			})
			.set_parent(contact)
			.id();

		match self.lifetime {
			LifeTimeData::UntilStopped => (contact, projection, OnSkillStop::Stop(contact)),
			LifeTimeData::UntilOutlived(duration) => {
				commands.try_insert_on(contact, LifeTime(duration));
				(contact, projection, OnSkillStop::Ignore)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use behaviors::components::{ground_targeted_aoe::GroundTargetedAoe, LifeTime};
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
	) -> impl Fn(Commands) -> (Entity, Entity, OnSkillStop) {
		move |mut commands| gt.apply(&mut commands, &caster, &spawn, &target)
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_ground_target() {
		let mut app = setup();
		let caster_transform = Transform::from_xyz(1., 2., 3.);
		let target_ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));

		let (contact, ..) = app.world_mut().run_system_once(ground_target(
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
			Some(&GroundTargetedAoe::<Contact> {
				caster: caster_transform,
				target_ray,
				max_range: Units::new(20.),
				radius: Units::new(8.),
				..default()
			}),
			app.world()
				.entity(contact)
				.get::<GroundTargetedAoe<Contact>>()
		)
	}

	#[test]
	fn spawn_as_stoppable() {
		let mut app = setup();

		let (contact, projection, on_skill_stop) = app.world_mut().run_system_once(ground_target(
			SpawnGroundTargetedAoe {
				lifetime: LifeTimeData::UntilStopped,
				..default()
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::from(GroundTargetedAoe::DEFAULT_TARGET_RAY),
		));

		assert_eq!(OnSkillStop::Stop(contact), on_skill_stop);
	}

	#[test]
	fn spawn_as_non_stoppable() {
		let mut app = setup();

		let (contact, projection, on_skill_stop) = app.world_mut().run_system_once(ground_target(
			SpawnGroundTargetedAoe {
				lifetime: LifeTimeData::UntilOutlived(Duration::from_micros(33)),
				..default()
			},
			SkillCaster::from(Entity::from_raw(42)),
			SkillSpawner::from(Entity::from_raw(43)),
			Target::from(GroundTargetedAoe::DEFAULT_TARGET_RAY),
		));

		assert_eq!(
			(
				OnSkillStop::Ignore,
				Some(&LifeTime(Duration::from_micros(33)))
			),
			(on_skill_stop, app.world().entity(contact).get::<LifeTime>())
		);
	}
}
