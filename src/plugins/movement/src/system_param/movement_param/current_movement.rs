use crate::system_param::movement_param::{MovementContext, MovementContextMut};
use common::traits::handles_movement::{CurrentMovement, MovementTarget};

impl CurrentMovement for MovementContext<'_> {
	fn current_movement(&self) -> Option<MovementTarget> {
		match self {
			MovementContext::Movement(movement) => movement.current_movement(),
			_ => None,
		}
	}
}

impl CurrentMovement for MovementContextMut<'_> {
	fn current_movement(&self) -> Option<MovementTarget> {
		self.movement
			.and_then(|movement| movement.current_movement())
	}
}
