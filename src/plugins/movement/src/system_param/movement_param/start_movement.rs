use crate::{components::movement::Movement, system_param::movement_param::MovementContextMut};
use bevy::ecs::component::Component;
use common::traits::handles_movement::{MovementTarget, StartMovement};

const ALLOWED_ANGLE: f32 = 1_f32.to_radians();

impl<TMotion> StartMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn start<T>(&mut self, target: T)
	where
		T: Into<MovementTarget>,
	{
		let movement = Movement::from(target);

		if directions_within_tolerance(self.movement, &movement) {
			return;
		}

		self.entity.try_insert(movement);
	}
}

fn directions_within_tolerance(l: Option<&Movement>, r: &Movement) -> bool {
	let (Some(Movement::Direction(l)), Movement::Direction(r)) = (l, r) else {
		return false;
	};

	l.angle_between(**r) <= ALLOWED_ANGLE
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::config::Config, system_param::movement_param::MovementParamMut};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::TryGetContextMut,
		handles_movement::ConfiguredMovement,
		thread_safe::ThreadSafe,
	};
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, IsChanged::<Movement>::detect);

		app
	}

	#[test_case(Vec3::new(1., 2., 3.); "to point")]
	#[test_case(Dir3::NEG_X; "towards direction")]
	fn insert_movement(
		target: impl Into<MovementTarget> + Copy + ThreadSafe,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::try_get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.start(target);
			})?;

		assert_eq!(
			Some(&Movement::from(target)),
			app.world().entity(entity).get::<Movement>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_insert_movement_direction_if_already_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Config::default(), Movement::from(Dir3::NEG_X)))
			.id();

		app.update();
		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::try_get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.start(Dir3::NEG_X);
			})?;
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Movement>>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_insert_movement_direction_if_already_present_within_tolerance()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let dir = Vec3::NEG_X.rotate_axis(Vec3::Y, ALLOWED_ANGLE);
		let dir = Dir3::try_from(dir).unwrap();
		let entity = app
			.world_mut()
			.spawn((Config::default(), Movement::from(dir)))
			.id();

		app.update();
		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::try_get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.start(Dir3::NEG_X);
			})?;
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Movement>>(),
		);
		Ok(())
	}

	#[test]
	fn re_insert_movement_direction_if_already_present_but_outside_tolerance()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let dir = Vec3::NEG_X.rotate_axis(Vec3::Y, ALLOWED_ANGLE * 1.1);
		let dir = Dir3::try_from(dir).unwrap();
		let entity = app
			.world_mut()
			.spawn((Config::default(), Movement::from(dir)))
			.id();

		app.update();
		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::try_get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.start(Dir3::NEG_X);
			})?;
		app.update();

		assert_eq!(
			Some(&Movement::from(Dir3::NEG_X)),
			app.world().entity(entity).get::<Movement>(),
		);
		Ok(())
	}
}
