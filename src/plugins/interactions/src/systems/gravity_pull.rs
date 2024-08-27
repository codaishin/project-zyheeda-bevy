use crate::components::effected_by_gravity::EffectedByGravity;
use bevy::prelude::{Bundle, Commands, Entity, Query, Transform};
use bevy_rapier3d::prelude::Velocity;
use common::{components::Immobilized, traits::try_insert_on::TryInsertOn};

pub(crate) fn gravity_pull(
	mut commands: Commands,
	mut agents: Query<(Entity, &Transform, &mut EffectedByGravity)>,
) {
	for (entity, transform, mut effected_by_gravity) in &mut agents {
		let position = transform.translation;
		let velocity = effected_by_gravity
			.pulls
			.drain(..)
			.map(|pull| (pull.towards - position).normalize() * *pull.strength)
			.reduce(|a, b| a + b)
			.map(Velocity::linear);

		let Some(velocity) = velocity else {
			continue;
		};

		commands.try_insert_on(entity, ForcedMovement::new(velocity));
	}
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
	use crate::components::effected_by_gravity::Pull;

	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
		prelude::Transform,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, gravity_pull);

		app
	}

	#[test]
	fn add_force_movement_for_single_pull() {
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

		app.update();

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
	fn add_force_movement_for_multiple_pulls() {
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

		app.update();

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

		app.update();

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

		app.update();

		let agent = app.world().entity(agent);
		assert_eq!(
			Some(&EffectedByGravity { pulls: vec![] }),
			agent.get::<EffectedByGravity>()
		)
	}
}
