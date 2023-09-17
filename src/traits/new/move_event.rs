use super::New1;
use crate::events::MoveEvent;
use bevy::prelude::Vec3;

impl New1<Vec3> for MoveEvent {
	fn new(target: Vec3) -> Self {
		Self { target }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_vector() {
		let event = MoveEvent::new(Vec3::ONE);

		assert_eq!(Vec3::ONE, event.target);
	}
}
