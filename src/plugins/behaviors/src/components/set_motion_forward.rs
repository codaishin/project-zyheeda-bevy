use bevy::prelude::*;
use common::{
	tools::{UnitsPerSecond, speed::Speed},
	traits::{accessors::get::TryApplyOn, handles_physics::LinearMotion},
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
		TMotion: From<LinearMotion> + Component,
	{
		for (entity, SetMotionForward(speed), transform) in &set_velocities {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(TMotion::from(LinearMotion::Direction {
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
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{
		handles_physics::LinearMotion,
		register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::{ApproxEqual, SingleThreadedApp, assert_eq_approx};

	#[derive(Debug, PartialEq)]
	struct _Movement;

	#[derive(Component, Debug, PartialEq)]
	struct _Motion(LinearMotion);

	impl ApproxEqual<f32> for _Motion {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			match (self.0, other.0) {
				(
					LinearMotion::Direction {
						speed: speed_a,
						direction: dir_a,
					},
					LinearMotion::Direction {
						speed: speed_b,
						direction: dir_b,
					},
				) => {
					speed_a.approx_equal(&*speed_b, tolerance)
						&& dir_a.approx_equal(&dir_b, tolerance)
				}
				(
					LinearMotion::ToTarget {
						speed: speed_a,
						target: tgt_a,
					},
					LinearMotion::ToTarget {
						speed: speed_b,
						target: tgt_b,
					},
				) => {
					speed_a.approx_equal(&*speed_b, tolerance)
						&& tgt_a.approx_equal(&tgt_b, tolerance)
				}
				_ => false,
			}
		}
	}

	impl From<LinearMotion> for _Motion {
		fn from(linear: LinearMotion) -> Self {
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
			Some(&_Motion(LinearMotion::Direction {
				speed: Speed(UnitsPerSecond::from(1.)),
				direction: Dir3::try_from(Vec3::new(1., 2., 3.).normalize()).unwrap()
			})),
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
			Some(&_Motion(LinearMotion::Direction {
				speed: Speed(UnitsPerSecond::from(10.)),
				direction: Dir3::try_from(Vec3::new(1., 2., 3.)).unwrap()
			})),
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
