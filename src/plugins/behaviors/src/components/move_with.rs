use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct MoveWith {
	pub(crate) entity: Entity,
}

impl MoveWith {
	pub(crate) fn set_position(
		mut force_shields: Query<(&Self, &mut Transform)>,
		transforms: Query<&GlobalTransform>,
	) {
		for (move_with, mut transform) in &mut force_shields {
			let Ok(location) = transforms.get(move_with.entity) else {
				continue;
			};
			*transform = Transform::from(*location).with_scale(Vec3::ONE);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, MoveWith::set_position);

		app
	}

	#[test]
	fn copy_location_translation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let agent = app
			.world_mut()
			.spawn((MoveWith { entity }, Transform::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn copy_location_rotation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((MoveWith { entity }, Transform::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn set_scale_to_one() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().with_scale(Vec3::new(3., 4., 5.)),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((MoveWith { entity }, Transform::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::default().with_scale(Vec3::ONE)),
			app.world().entity(agent).get::<Transform>()
		);
	}
}
