use crate::components::gravity_affected::{GravityAffected, GravityPull};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::immobilized::Immobilized,
	traits::accessors::get::{GetMut, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};
use std::time::Duration;

pub(crate) fn apply_gravity_pull(
	In(delta): In<Duration>,
	mut commands: ZyheedaCommands,
	mut agents: Query<(Entity, &Transform, &mut GravityAffected)>,
	transforms: Query<&GlobalTransform>,
) {
	let delta_secs = delta.as_secs_f32();

	for (entity, transform, mut gravity_affected) in &mut agents {
		if gravity_affected.is_not_pulled() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Immobilized>();
			});
			continue;
		}

		let mut get_pull_center = |pull: &GravityPull| {
			let towards = commands.get_mut(&pull.towards)?.id();
			let translation = transforms.get(towards).ok()?.translation();
			Some(translation.with_y(0.))
		};
		let position = transform.translation.with_y(0.);
		let mut pull_sum = None;

		for pull in gravity_affected.drain_pulls(..) {
			let Some(pull_center) = get_pull_center(&pull) else {
				continue;
			};
			let Some(pull_vector) = get_pull_vector(delta_secs, position, pull, pull_center) else {
				continue;
			};

			*pull_sum.get_or_insert(Vec3::ZERO) += pull_vector;
		}

		let Some(pull_sum) = pull_sum else {
			continue;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert((Immobilized, Velocity::linear(pull_sum)));
		});
	}
}

fn get_pull_vector(
	delta_secs: f32,
	position: Vec3,
	pull: GravityPull,
	pull_center: Vec3,
) -> Option<Vec3> {
	let direction = pull_center - position;

	match predict(direction, *pull.strength, delta_secs) {
		Predict::Overshoot => Some(direction / delta_secs),
		Predict::NormalAdvance => Some(direction.normalize_or_zero() * *pull.strength),
	}
}

enum Predict {
	Overshoot,
	NormalAdvance,
}

fn predict(direction: Vec3, pull_strength: f32, delta_secs: f32) -> Predict {
	let movement_in_one_frame = pull_strength * delta_secs;
	if direction.length() < movement_in_one_frame {
		return Predict::Overshoot;
	}

	Predict::NormalAdvance
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::gravity_affected::GravityPull;
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce},
		math::Vec3,
		prelude::Transform,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::UnitsPerSecond,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn add_forced_movement_for_single_pull() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 3.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(2.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-1., 0., 3.).normalize() * 2.)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn add_forced_movement_for_single_pull_and_put_gravity_center_at_zero_elevation()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 10., 3.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(2.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-1., 0., 3.).normalize() * 2.)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn add_forced_movement_for_multiple_pulls() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards_a = PersistentEntity::default();
		app.world_mut().spawn((
			towards_a,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 3.))),
		));
		let towards_b = PersistentEntity::default();
		app.world_mut().spawn((
			towards_b,
			GlobalTransform::from(Transform::from_translation(Vec3::new(-2., 0., 0.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([
					GravityPull {
						strength: UnitsPerSecond::from(2.),
						towards: towards_a,
					},
					GravityPull {
						strength: UnitsPerSecond::from(3.),
						towards: towards_b,
					},
				]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(
					(Vec3::new(-1., 0., 3.).normalize() * 2.)
						+ (Vec3::new(-3., 0., 0.).normalize() * 3.)
				)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn do_not_add_forced_movement_if_pulls_array_empty() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(1., 0., 0.), GravityAffected::default()))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(None, None),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn empty_pulls_array() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards_a = PersistentEntity::default();
		app.world_mut().spawn((
			towards_a,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 3.))),
		));
		let towards_b = PersistentEntity::default();
		app.world_mut().spawn((
			towards_b,
			GlobalTransform::from(Transform::from_translation(Vec3::new(-2., 0., 0.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([
					GravityPull {
						strength: UnitsPerSecond::from(2.),
						towards: towards_a,
					},
					GravityPull {
						strength: UnitsPerSecond::from(3.),
						towards: towards_b,
					},
				]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			Some(&GravityAffected::default()),
			agent.get::<GravityAffected>()
		);
		Ok(())
	}

	#[test]
	fn remove_immobilized_if_pulls_empty() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				Immobilized,
				GravityAffected::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_secs(1))?;

		let agent = app.world().entity(agent);
		assert_eq!(None, agent.get::<Immobilized>());
		Ok(())
	}

	#[test]
	fn use_direction_length_divided_by_delta_when_pull_times_delta_exceed_direction_length()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let delta = Duration::from_millis(501);
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 4.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(10.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, delta)?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(
					Vec3::new(-3., 0., 4.) / delta.as_secs_f32()
				)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn use_pull_strength_when_pull_times_delta_do_not_exceed_direction_length()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 4.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(10.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_millis(499))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-3., 0., 4.).normalize() * 10.)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn compute_direction_from_zero_elevation_of_agent() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(0., 0., 4.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 1., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(1.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::from_millis(499))?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-3., 0., 4.).normalize())),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}

	#[test]
	fn use_pull_strength_zero_when_delta_zero_and_direction_zero() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = PersistentEntity::default();
		app.world_mut().spawn((
			towards,
			GlobalTransform::from(Transform::from_translation(Vec3::new(3., 0., 0.))),
		));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::from(10.),
					towards,
				}]),
			))
			.id();

		app.world_mut()
			.run_system_once_with(apply_gravity_pull, Duration::ZERO)?;

		let agent = app.world().entity(agent);
		assert_eq!(
			(Some(&Velocity::linear(Vec3::ZERO)), Some(&Immobilized)),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		);
		Ok(())
	}
}
