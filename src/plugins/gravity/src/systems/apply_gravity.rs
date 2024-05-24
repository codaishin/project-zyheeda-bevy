use crate::components::Gravity;
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query},
	},
	math::Vec3,
	transform::components::Transform,
};
use bevy_rapier3d::dynamics::Velocity;
use common::traits::try_insert_on::TryInsertOn;

const SENSITIVITY: f32 = 0.1;

pub(crate) fn apply_gravity(
	mut commands: Commands,
	agents: Query<(Entity, &Transform, &Gravity)>,
	transforms: Query<&Transform>,
) {
	for (entity, transform, gravity) in &agents {
		let Some(direction) = get_direction(transform, gravity, &transforms) else {
			continue;
		};
		let speed = *gravity.pull;
		let velocity = match direction.length() < SENSITIVITY * speed {
			true => Velocity::default(),
			false => Velocity::linear(direction.normalize() * speed),
		};

		commands.try_insert_on(entity, velocity);
	}
}

fn get_direction(
	transform: &Transform,
	gravity: &Gravity,
	transforms: &Query<&Transform, ()>,
) -> Option<Vec3> {
	let center = transforms.get(gravity.center).ok()?;
	Some(
		center.translation
			- Vec3::new(
				transform.translation.x,
				center.translation.y,
				transform.translation.z,
			),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
	};
	use bevy_rapier3d::dynamics::Velocity;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, apply_gravity);

		app
	}

	#[test]
	fn apply_gravity_pull_to_global_center() {
		let mut app = setup();
		let center = app.world.spawn(Transform::default()).id();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(1., 0., 0.),
				Gravity {
					pull: UnitsPerSecond::new(1.),
					center,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(-1., 0., 0.))),
			agent.get::<Velocity>()
		);
	}

	#[test]
	fn apply_gravity_pull_to_arbitrary_location() {
		let mut app = setup();
		let center = app.world.spawn(Transform::from_xyz(1., 2., 1.)).id();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(2., 2., 2.),
				Gravity {
					pull: UnitsPerSecond::new(1.),
					center,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(-1., 0., -1.).normalize())),
			agent.get::<Velocity>()
		);
	}

	#[test]
	fn scale_gravity_pull_by_configured_units_per_second() {
		let mut app = setup();
		let center = app.world.spawn(Transform::from_xyz(1., 2., 1.)).id();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(2., 2., 2.),
				Gravity {
					pull: UnitsPerSecond::new(0.5),
					center,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(-1., 0., -1.).normalize() * 0.5)),
			agent.get::<Velocity>()
		);
	}

	#[test]
	fn move_only_horizontally() {
		let mut app = setup();
		let center = app.world.spawn(Transform::from_xyz(1., 3., 1.)).id();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(2., 2., 2.),
				Gravity {
					pull: UnitsPerSecond::new(1.),
					center,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(-1., 0., -1.).normalize())),
			agent.get::<Velocity>()
		);
	}

	#[test]
	fn do_not_move_when_within_sensitivity_range_to_center_scaled_by_pull() {
		let mut app = setup();
		let center = app.world.spawn(Transform::from_xyz(0., 0., 0.)).id();
		let agent = app
			.world
			.spawn((
				Transform::from_translation(Vec3::new((SENSITIVITY - f32::EPSILON) * 10., 0., 0.)),
				Gravity {
					pull: UnitsPerSecond::new(10.),
					center,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&Velocity::default()), agent.get::<Velocity>());
	}
}
