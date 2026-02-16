use crate::components::{set_velocity_forward::SetVelocityForward, velocity::LinearVelocity};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SetVelocityForward {
	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		set_velocities: Query<(Entity, &Self, &Transform)>,
	) {
		for (entity, SetVelocityForward(speed), transform) in &set_velocities {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(LinearVelocity(*transform.forward() * **speed));
				e.try_remove::<SetVelocityForward>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::{SingleThreadedApp, assert_eq_approx};

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
				SetVelocityForward(UnitsPerSecond::from(1.)),
			))
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq_approx!(
			Some(&LinearVelocity(Vec3::new(1., 2., 3.).normalize())),
			app.world().entity(entity).get::<LinearVelocity>(),
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
				SetVelocityForward(UnitsPerSecond::from(10.)),
			))
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq_approx!(
			Some(&LinearVelocity(Vec3::new(1., 2., 3.).normalize() * 10.)),
			app.world().entity(entity).get::<LinearVelocity>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn remove_velocity_setter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward(UnitsPerSecond::from(1.)))
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system)?;

		assert_eq!(None, app.world().entity(entity).get::<SetVelocityForward>());
		Ok(())
	}
}
