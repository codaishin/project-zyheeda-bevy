use crate::events::MoveEvent;
use bevy::prelude::Vec3;

impl From<Vec3> for MoveEvent {
	fn from(target: Vec3) -> Self {
		Self { target }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_vec3() {
		let vec = Vec3::new(1., 2., 3.);
		let event = MoveEvent::from(vec);

		assert_eq!(vec, event.target);
	}
}
