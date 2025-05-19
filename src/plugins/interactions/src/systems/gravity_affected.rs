use crate::components::gravity_affected::{GravityAffected, GravityPull};
use bevy::{
	math::Vec3,
	prelude::{Bundle, Commands, Entity, GlobalTransform, In, Query, Transform},
};
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::Immobilized,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use std::time::Duration;

pub(crate) fn apply_gravity_pull(
	In(delta): In<Duration>,
	mut commands: Commands,
	mut agents: Query<(Entity, &Transform, &mut GravityAffected)>,
	transforms: Query<&GlobalTransform>,
) {
	let translation = |entity| {
		let transform = transforms.get(entity).ok()?;
		Some(transform.translation())
	};

	for (entity, transform, mut gravity_affected) in &mut agents {
		if gravity_affected.is_not_pulled() {
			commands.try_remove_from::<Immobilized>(entity);
			continue;
		}

		let position = transform.translation;
		let velocity = gravity_affected
			.drain_pulls(..)
			.filter_map(velocity_vector(position, delta, pull_towards(translation)))
			.reduce(sum)
			.map(Velocity::linear);

		let Some(velocity) = velocity else {
			continue;
		};

		commands.try_insert_on(entity, ForcedMovement::new(velocity));
	}
}

fn velocity_vector(
	position: Vec3,
	delta: Duration,
	pull_towards: impl Fn(&GravityPull) -> Option<Vec3>,
) -> impl Fn(GravityPull) -> Option<Vec3> {
	move |pull| {
		let direction = pull_towards(&pull)? - position;
		let pull_strength = *pull.strength;
		let delta_secs = delta.as_secs_f32();

		match predict(direction, pull_strength, delta_secs) {
			Predict::Overshoot => Some(direction / delta_secs),
			Predict::NormalAdvance => Some(direction.normalize() * pull_strength),
		}
	}
}

fn pull_towards(
	translation: impl Fn(Entity) -> Option<Vec3>,
) -> impl Fn(&GravityPull) -> Option<Vec3> {
	move |GravityPull { towards, .. }: &GravityPull| {
		let translation = translation(*towards)?;
		Some(Vec3::new(translation.x, 0., translation.z))
	}
}

fn sum(a: Vec3, b: Vec3) -> Vec3 {
	a + b
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

#[derive(Bundle)]
struct ForcedMovement(Velocity, Immobilized);

impl ForcedMovement {
	fn new(velocity: Velocity) -> Self {
		ForcedMovement(velocity, Immobilized)
	}
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
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn add_forced_movement_for_single_pull() -> Result<(), RunSystemError> {
		let mut app = setup();
		let towards = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 0., 3.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::new(2.),
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
		let towards = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 10., 3.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::new(2.),
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
		let towards_a = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 0., 3.),
			)))
			.id();
		let towards_b = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(-2., 0., 0.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([
					GravityPull {
						strength: UnitsPerSecond::new(2.),
						towards: towards_a,
					},
					GravityPull {
						strength: UnitsPerSecond::new(3.),
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
		let towards_a = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 0., 3.),
			)))
			.id();
		let towards_b = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(-2., 0., 0.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				GravityAffected::new([
					GravityPull {
						strength: UnitsPerSecond::new(2.),
						towards: towards_a,
					},
					GravityPull {
						strength: UnitsPerSecond::new(3.),
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
		let towards = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 0., 4.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::new(10.),
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
		let towards = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::from_translation(
				Vec3::new(0., 0., 4.),
			)))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				GravityAffected::new([GravityPull {
					strength: UnitsPerSecond::new(10.),
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
}
