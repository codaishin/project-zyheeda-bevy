use crate::components::{motion::Motion, set_motion_forward::SetMotionForward};
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::{accessors::get::TryApplyOn, handles_physics::LinearMotion},
	zyheeda_commands::ZyheedaCommands,
};

impl SetMotionForward {
	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		set_velocities: Query<(Entity, &Self, &Transform)>,
	) {
		for (entity, SetMotionForward(speed), transform) in &set_velocities {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Motion::from(LinearMotion::Direction {
					speed: Speed(*speed),
					direction: transform.forward(),
				}));
				e.try_remove::<SetMotionForward>();
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
		traits::{
			handles_physics::LinearMotion,
			register_persistent_entities::RegisterPersistentEntities,
		},
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
				SetMotionForward(UnitsPerSecond::from(1.)),
			))
			.id();

		app.world_mut().run_system_once(SetMotionForward::system)?;

		assert_eq_approx!(
			Some(&Motion::from(LinearMotion::Direction {
				speed: Speed(UnitsPerSecond::from(1.)),
				direction: Dir3::try_from(Vec3::new(1., 2., 3.).normalize()).unwrap()
			})),
			app.world().entity(entity).get::<Motion>(),
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
				SetMotionForward(UnitsPerSecond::from(10.)),
			))
			.id();

		app.world_mut().run_system_once(SetMotionForward::system)?;

		assert_eq_approx!(
			Some(&Motion::from(LinearMotion::Direction {
				speed: Speed(UnitsPerSecond::from(10.)),
				direction: Dir3::try_from(Vec3::new(1., 2., 3.)).unwrap()
			})),
			app.world().entity(entity).get::<Motion>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn remove_velocity_setter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SetMotionForward(UnitsPerSecond::from(1.)))
			.id();

		app.world_mut().run_system_once(SetMotionForward::system)?;

		assert_eq!(None, app.world().entity(entity).get::<SetMotionForward>());
		Ok(())
	}
}
