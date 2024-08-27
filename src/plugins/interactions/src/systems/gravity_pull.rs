use crate::components::effected_by_gravity::{EffectedByGravity, Pull};
use bevy::{
	math::Vec3,
	prelude::{Bundle, Commands, Entity, In, Query, Transform},
};
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::Immobilized,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use std::time::Duration;

pub(crate) fn gravity_pull(
	In(delta): In<Duration>,
	mut commands: Commands,
	mut agents: Query<(Entity, &Transform, &mut EffectedByGravity)>,
) {
	for (entity, transform, mut effected_by_gravity) in &mut agents {
		if effected_by_gravity.pulls.is_empty() {
			commands.try_remove_from::<Immobilized>(entity);
			continue;
		}

		let position = transform.translation;
		let velocity = effected_by_gravity
			.pulls
			.drain(..)
			.map(get_pull_vector(position, delta))
			.reduce(|a, b| a + b)
			.map(Velocity::linear);

		let Some(velocity) = velocity else {
			continue;
		};

		commands.try_insert_on(entity, ForcedMovement::new(velocity));
	}
}

fn get_pull_vector(position: Vec3, delta: Duration) -> impl Fn(Pull) -> Vec3 {
	move |pull| {
		let direction = pull.towards - position;
		let pull_strength = *pull.strength;
		if would_overshoot(direction, pull_strength, delta) {
			direction
		} else {
			direction.normalize() * pull_strength
		}
	}
}

fn would_overshoot(direction: Vec3, pull_strength: f32, delta: Duration) -> bool {
	direction.length() < pull_strength * delta.as_secs_f32()
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
	use crate::components::effected_by_gravity::Pull;
	use bevy::{app::App, ecs::system::RunSystemOnce, math::Vec3, prelude::Transform};
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn add_forced_movement_for_single_pull() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				EffectedByGravity {
					pulls: vec![Pull {
						strength: UnitsPerSecond::new(2.),
						towards: Vec3::new(0., 0., 3.),
					}],
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_secs(1), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-1., 0., 3.).normalize() * 2.)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		)
	}

	#[test]
	fn add_forced_movement_for_multiple_pulls() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				EffectedByGravity {
					pulls: vec![
						Pull {
							strength: UnitsPerSecond::new(2.),
							towards: Vec3::new(0., 0., 3.),
						},
						Pull {
							strength: UnitsPerSecond::new(3.),
							towards: Vec3::new(-2., 0., 0.),
						},
					],
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_secs(1), gravity_pull);

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
		)
	}

	#[test]
	fn do_not_add_forced_movement_if_pulls_array_empty() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				EffectedByGravity { pulls: vec![] },
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_secs(1), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(
			(None, None),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		)
	}

	#[test]
	fn empty_pulls_array() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				EffectedByGravity {
					pulls: vec![
						Pull {
							strength: UnitsPerSecond::new(2.),
							towards: Vec3::new(0., 0., 3.),
						},
						Pull {
							strength: UnitsPerSecond::new(3.),
							towards: Vec3::new(-2., 0., 0.),
						},
					],
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_secs(1), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(
			Some(&EffectedByGravity { pulls: vec![] }),
			agent.get::<EffectedByGravity>()
		)
	}

	#[test]
	fn remove_immobilized_if_pulls_empty() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				Immobilized,
				EffectedByGravity { pulls: vec![] },
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_secs(1), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(None, agent.get::<Immobilized>())
	}

	#[test]
	fn use_direction_length_when_pull_times_delta_exceed_direction_length() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				EffectedByGravity {
					pulls: vec![Pull {
						strength: UnitsPerSecond::new(10.),
						towards: Vec3::new(0., 0., 4.),
					}],
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(501), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-3., 0., 4.))),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		)
	}

	#[test]
	fn use_pull_strength_when_pull_times_delta_do_not_exceed_direction_length() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(3., 0., 0.),
				EffectedByGravity {
					pulls: vec![Pull {
						strength: UnitsPerSecond::new(10.),
						towards: Vec3::new(0., 0., 4.),
					}],
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(499), gravity_pull);

		let agent = app.world().entity(agent);
		assert_eq!(
			(
				Some(&Velocity::linear(Vec3::new(-3., 0., 4.).normalize() * 10.)),
				Some(&Immobilized)
			),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		)
	}
}
