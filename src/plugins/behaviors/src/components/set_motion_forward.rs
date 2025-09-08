use bevy::prelude::*;
use common::{
	tools::UnitsPerSecond,
	traits::{accessors::get::TryApplyOn, handles_physics::Linear},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform)]
pub(crate) struct SetMotionForward(pub(crate) UnitsPerSecond);

impl SetMotionForward {
	pub(crate) fn system<TMotion>(
		mut commands: ZyheedaCommands,
		set_velocities: Query<(Entity, &Self, &Transform)>,
	) where
		TMotion: From<Linear> + Component,
	{
		for (entity, SetMotionForward(speed), transform) in &set_velocities {
			let movement = transform.forward() * **speed;
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(TMotion::from(Linear(movement)));
				e.try_remove::<SetMotionForward>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{
		handles_physics::Linear,
		register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::{ApproxEqual, SingleThreadedApp, assert_eq_approx};

	#[derive(Debug, PartialEq)]
	struct _Movement;

	#[derive(Component, Debug, PartialEq)]
	struct _Motion(Linear);

	impl ApproxEqual<f32> for _Motion {
		fn approx_equal(&self, Self(Linear(other)): &Self, tolerance: &f32) -> bool {
			let Self(Linear(this)) = self;
			this.approx_equal(other, tolerance)
		}
	}

	impl From<Linear> for _Motion {
		fn from(linear: Linear) -> Self {
			_Motion(linear)
		}
	}

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

		app.world_mut()
			.run_system_once(SetMotionForward::system::<_Motion>)?;

		assert_eq_approx!(
			Some(&_Motion(Linear(Vec3::new(1., 2., 3.).normalize()))),
			app.world().entity(entity).get::<_Motion>(),
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

		app.world_mut()
			.run_system_once(SetMotionForward::system::<_Motion>)?;

		assert_eq_approx!(
			Some(&_Motion(Linear(Vec3::new(1., 2., 3.).normalize() * 10.))),
			app.world().entity(entity).get::<_Motion>(),
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

		app.world_mut()
			.run_system_once(SetMotionForward::system::<_Motion>)?;

		assert_eq!(None, app.world().entity(entity).get::<SetMotionForward>());
		Ok(())
	}
}
