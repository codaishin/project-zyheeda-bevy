use crate::system_param::movement_param::{MovementContext, MovementContextMut};
use common::traits::handles_movement::{CurrentMovement, MovementTarget};

impl<TMotion, TImmobilized> CurrentMovement for MovementContext<'_, TMotion, TImmobilized> {
	fn current_movement(&self) -> Option<MovementTarget> {
		match self {
			MovementContext::Movement(movement) => movement.target,
			_ => None,
		}
	}
}

impl<TMotion, TImmobilized> CurrentMovement for MovementContextMut<'_, TMotion, TImmobilized> {
	fn current_movement(&self) -> Option<MovementTarget> {
		self.movement.and_then(|movement| movement.target)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::movement::{Movement, path_or_direction::PathOrDirection},
		system_param::movement_param::{
			MovementParam,
			MovementParamMut,
			context_changed::JustRemovedMovements,
		},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		math::Vec3,
	};
	use common::traits::{
		accessors::get::{GetContext, GetContextMut},
		handles_movement::Movement as MovementMarker,
	};
	use testing::SingleThreadedApp;

	struct _Immobilized;

	struct _Motion;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<JustRemovedMovements>();

		app
	}

	#[test]
	fn return_current_movement_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<PathOrDirection<_Motion>, _Immobilized>::to(
				Vec3::new(1., 2., 3.),
			))
			.id();
		app.world_mut()
			.run_system_once(move |m: MovementParam<_Motion, _Immobilized>| {
				let ctx = MovementParam::get_context(&m, MovementMarker { entity }).unwrap();

				assert_eq!(
					Some(MovementTarget::Point(Vec3::new(1., 2., 3.))),
					ctx.current_movement(),
				);
			})
	}

	#[test]
	fn return_current_movement_target_mut() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<PathOrDirection<_Motion>, _Immobilized>::to(
				Vec3::new(1., 2., 3.),
			))
			.id();
		app.world_mut()
			.run_system_once(move |mut m: MovementParamMut<_Motion, _Immobilized>| {
				let ctx =
					MovementParamMut::get_context_mut(&mut m, MovementMarker { entity }).unwrap();

				assert_eq!(
					Some(MovementTarget::Point(Vec3::new(1., 2., 3.))),
					ctx.current_movement(),
				);
			})
	}
}
