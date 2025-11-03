use crate::system_param::movement_param::MovementContextMut;
use common::traits::{
	handles_movement::{CurrentMovement, MovementTarget},
	thread_safe::ThreadSafe,
};

impl<TMotion> CurrentMovement for MovementContextMut<'_, TMotion>
where
	TMotion: ThreadSafe,
{
	fn current_movement(&self) -> Option<MovementTarget> {
		self.movement.and_then(|movement| movement.target)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::movement::{Movement, path_or_direction::PathOrDirection},
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		math::Vec3,
	};
	use common::traits::{
		accessors::get::EntityContextMut,
		handles_movement::Movement as MovementMarker,
	};
	use testing::SingleThreadedApp;

	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_current_movement_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<PathOrDirection<_Motion>>::to(Vec3::new(
				1., 2., 3.,
			)))
			.id();
		app.world_mut()
			.run_system_once(move |mut m: MovementParamMut<_Motion>| {
				let ctx = MovementParamMut::get_entity_context_mut(&mut m, entity, MovementMarker)
					.unwrap();

				assert_eq!(
					Some(MovementTarget::Point(Vec3::new(1., 2., 3.))),
					ctx.current_movement(),
				);
			})
	}
}
