use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	tools::UnitsPerSecond,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform)]
pub(crate) struct SetVelocityForward(pub(crate) UnitsPerSecond);

impl SetVelocityForward {
	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		set_velocities: Query<(Entity, &Self, &Transform)>,
	) {
		for (entity, SetVelocityForward(speed), transform) in &set_velocities {
			let movement = transform.forward() * **speed;
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Velocity::linear(movement));
				e.try_remove::<SetVelocityForward>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::Velocity;
	use common::traits::{
		clamp_zero_positive::ClampZeroPositive,
		register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::{SingleThreadedApp, assert_eq_approx};

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
		let entity = app
			.world_mut()
			.spawn((
				Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y),
				SetVelocityForward(UnitsPerSecond::new(1.)),
			))
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
		let entity = app
			.world_mut()
			.spawn((
				Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y),
				SetVelocityForward(UnitsPerSecond::new(10.)),
			))
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
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward(UnitsPerSecond::new(1.)))
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq!(None, app.world().entity(entity).get::<SetVelocityForward>());
		Ok(())
	}
}
