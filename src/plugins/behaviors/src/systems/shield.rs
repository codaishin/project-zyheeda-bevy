use crate::components::shield::Shield;
use bevy::{
	ecs::system::Query,
	math::Vec3,
	transform::components::{GlobalTransform, Transform},
};

pub(crate) fn position_force_shield(
	mut force_shields: Query<(&Shield, &mut Transform)>,
	transforms: Query<&GlobalTransform>,
) {
	for (force_shield, mut transform) in &mut force_shields {
		let Ok(location) = transforms.get(force_shield.location) else {
			continue;
		};
		*transform = Transform::from(*location).with_scale(Vec3::ONE);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, position_force_shield);

		app
	}

	#[test]
	fn copy_location_translation() {
		let mut app = setup();
		let location = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let agent = app
			.world_mut()
			.spawn((Shield { location }, Transform::default()))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn copy_location_rotation() {
		let mut app = setup();
		let location = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((Shield { location }, Transform::default()))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn set_scale_to_one() {
		let mut app = setup();
		let location = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().with_scale(Vec3::new(3., 4., 5.)),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((Shield { location }, Transform::default()))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::default().with_scale(Vec3::ONE)),
			agent.get::<Transform>()
		);
	}
}
