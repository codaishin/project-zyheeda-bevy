use crate::{
	components::movement::{Movement, path_or_direction::PathOrDirection},
	system_param::movement_param::MovementContextMut,
};
use common::traits::{handles_movement::StopMovement, thread_safe::ThreadSafe};

impl<TMotion> StopMovement for MovementContextMut<'_, TMotion>
where
	TMotion: ThreadSafe,
{
	fn stop(&mut self) {
		self.entity
			.try_insert(Movement::<PathOrDirection<TMotion>>::stop());
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::movement::{Movement, path_or_direction::PathOrDirection},
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
	fn insert_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<PathOrDirection<_Motion>>::to(Vec3::new(
				1., 2., 3.,
			)))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, MovementMarker { entity }).unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&Movement::stop()),
			app.world()
				.entity(entity)
				.get::<Movement<PathOrDirection<_Motion>>>()
		);
		Ok(())
	}
}
