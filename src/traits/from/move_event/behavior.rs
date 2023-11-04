use crate::{behaviors::Behavior, events::MoveEvent};

impl From<MoveEvent> for Behavior {
	fn from(event: MoveEvent) -> Self {
		Self::MoveTo(event.target)
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;

	#[test]
	fn move_to() {
		let event = MoveEvent {
			target: Vec3::new(3., 2., 1.),
		};
		let behavior: Behavior = event.into();

		assert_eq!(Behavior::MoveTo(Vec3::new(3., 2., 1.)), behavior);
	}
}
