use bevy::prelude::Vec3;

use crate::events::MoveEnqueueEvent;

impl From<Vec3> for MoveEnqueueEvent {
	fn from(target: Vec3) -> Self {
		Self { target }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_vec3() {
		let vec = Vec3::new(2., 1., 3.);
		let event = MoveEnqueueEvent::from(vec);

		assert_eq!(vec, event.target);
	}
}
