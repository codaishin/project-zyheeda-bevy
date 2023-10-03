use crate::{behavior::SimpleMovement, events::MoveEnqueueEvent};

impl From<MoveEnqueueEvent> for SimpleMovement {
	fn from(event: MoveEnqueueEvent) -> Self {
		Self {
			target: Some(event.target),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::Vec3;

	#[test]
	fn from_move_event() {
		let event = MoveEnqueueEvent {
			target: Vec3::new(3., 4., 5.),
		};
		let movement = SimpleMovement::from(event);

		assert_eq!(Some(event.target), movement.target);
	}
}
