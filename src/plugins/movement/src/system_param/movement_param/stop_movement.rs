use crate::{
	components::{movement::path_or_direction::PathOrDirection, new_movement::NewMovement},
	system_param::movement_param::MovementContextMut,
};
use common::traits::handles_movement::StopMovement;

impl StopMovement for MovementContextMut<'_> {
	fn stop(&mut self) {
		self.entity
			.try_insert((PathOrDirection::stop(), NewMovement::Stopped));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::movement::path_or_direction::PathOrDirection,
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_movement::Movement as MovementMarker,
	};
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_path_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, MovementMarker { entity }).unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&PathOrDirection::stop()),
			app.world().entity(entity).get::<PathOrDirection>(),
		);
		Ok(())
	}

	#[test]
	fn insert_movement_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, MovementMarker { entity }).unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&NewMovement::Stopped),
			app.world().entity(entity).get::<NewMovement>(),
		);
		Ok(())
	}
}
