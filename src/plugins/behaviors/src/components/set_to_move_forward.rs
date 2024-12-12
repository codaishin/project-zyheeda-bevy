use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	tools::UnitsPerSecond,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct SetVelocityForward {
	pub(crate) rotation: Entity,
	pub(crate) speed: UnitsPerSecond,
}

impl SetVelocityForward {
	pub(crate) fn system(
		mut commands: Commands,
		set_velocities: Query<(Entity, &Self)>,
		transforms: Query<&Transform>,
	) {
		for (entity, set_velocity) in &set_velocities {
			let Ok(rotation) = transforms.get(set_velocity.rotation) else {
				continue;
			};
			let movement = rotation.forward() * *set_velocity.speed;
			commands.try_insert_on(entity, Velocity::linear(movement));
			commands.try_remove_from::<SetVelocityForward>(entity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::prelude::Velocity;
	use common::{
		assert_eq_approx,
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Debug, PartialEq)]
	struct _Movement;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SetVelocityForward::system);

		app
	}

	#[test]
	fn insert_velocity() {
		let mut app = setup();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation,
				speed: UnitsPerSecond::new(1.),
			})
			.id();

		app.update();

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize())),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
	}

	#[test]
	fn insert_velocity_scaled_by_speed() {
		let mut app = setup();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.update();

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize() * 10.)),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
	}

	#[test]
	fn remove_velocity_setter() {
		let mut app = setup();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetVelocityForward>());
	}
}
