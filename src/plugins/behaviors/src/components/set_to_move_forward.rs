use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::persistent_entity::PersistentEntity,
	resources::persistent_entities::PersistentEntities,
	tools::UnitsPerSecond,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct SetVelocityForward {
	pub(crate) rotation: PersistentEntity,
	pub(crate) speed: UnitsPerSecond,
}

impl SetVelocityForward {
	pub(crate) fn system(
		mut commands: Commands,
		mut persistent_entities: ResMut<PersistentEntities>,
		set_velocities: Query<(Entity, &Self)>,
		transforms: Query<&Transform>,
	) {
		for (entity, set_velocity) in &set_velocities {
			let Some(rotation) = persistent_entities.get_entity(&set_velocity.rotation) else {
				continue;
			};
			let Ok(rotation) = transforms.get(rotation) else {
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::Velocity;
	use common::{
		assert_eq_approx,
		test_tools::utils::SingleThreadedApp,
		traits::{
			clamp_zero_positive::ClampZeroPositive,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};

	#[derive(Debug, PartialEq)]
	struct _Movement;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn insert_velocity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = PersistentEntity::default();
		app.world_mut().spawn((
			Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y),
			target,
		));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: target,
				speed: UnitsPerSecond::new(1.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize())),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn insert_velocity_scaled_by_speed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = PersistentEntity::default();
		app.world_mut().spawn((
			Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y),
			target,
		));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: target,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize() * 10.)),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn remove_velocity_setter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = PersistentEntity::default();
		app.world_mut().spawn((
			Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y),
			target,
		));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: target,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq!(None, app.world().entity(entity).get::<SetVelocityForward>());
		Ok(())
	}
}
