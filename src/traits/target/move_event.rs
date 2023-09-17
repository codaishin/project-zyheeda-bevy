use super::Target;
use crate::events::MoveEvent;
use bevy::math::Vec3;

impl Target for MoveEvent {
	fn target(&self) -> Vec3 {
		self.target
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_target() {
		let event = MoveEvent {
			target: Vec3::new(1., 2., 3.),
		};
		assert_eq!(event.target, event.target());
	}
}
