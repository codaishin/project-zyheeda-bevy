use crate::{
	components::{
		movement_path::MovementPath,
		movement_definition::MovementDefinition,
		ongoing_movement::OngoingMovement,
	},
	system_param::movement_param::MovementContextMut,
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::handles_movement::{MovementTarget, StartMovement},
};

impl StartMovement for MovementContextMut<'_> {
	fn start<T>(&mut self, target: T, radius: Units, speed: UnitsPerSecond)
	where
		T: Into<MovementTarget>,
	{
		self.entity.try_insert((
			OngoingMovement::Stopped,
			MovementPath::from(target),
			MovementDefinition { radius, speed },
		));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			movement_path::MovementPath,
			movement_definition::MovementDefinition,
		},
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_movement::Movement,
		thread_safe::ThreadSafe,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, Movement { entity }).unwrap();
				ctx.start(
					Vec3::new(1., 2., 3.),
					Units::from(42.),
					UnitsPerSecond::from(11.),
				);
			})?;

		assert_eq!(
			Some(&MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(11.),
			}),
			app.world().entity(entity).get::<MovementDefinition>(),
		);
		Ok(())
	}

	#[test_case(Vec3::new(1.,2.,3.); "to point")]
	#[test_case(Dir3::NEG_X; "towards direction")]
	fn insert_path(
		target: impl Into<MovementTarget> + Copy + ThreadSafe,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, Movement { entity }).unwrap();
				ctx.start(target, Units::from(42.), UnitsPerSecond::from(11.));
			})?;

		assert_eq!(
			Some(&MovementPath::from(target)),
			app.world().entity(entity).get::<MovementPath>(),
		);
		Ok(())
	}

	#[test]
	fn insert_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, Movement { entity }).unwrap();
				ctx.start(
					Vec3::new(1., 2., 3.),
					Units::from(42.),
					UnitsPerSecond::from(11.),
				);
			})?;

		assert_eq!(
			Some(&OngoingMovement::Stopped),
			app.world().entity(entity).get::<OngoingMovement>(),
		);
		Ok(())
	}
}
