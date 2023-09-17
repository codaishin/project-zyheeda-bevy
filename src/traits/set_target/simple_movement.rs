use crate::components::SimpleMovement;

use super::SetTarget;

impl SetTarget for SimpleMovement {
	fn set_target(&mut self, target: Option<bevy::prelude::Vec3>) {
		if target.is_none() {
			return;
		}
		self.target = target;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::UnitsPerSecond;
	use bevy::prelude::Vec3;

	#[test]
	fn set_target() {
		let target = Some(Vec3::X);
		let mut movement = SimpleMovement {
			target: None,
			speed: UnitsPerSecond::new(0.),
		};

		movement.set_target(target);

		assert_eq!(target, movement.target);
	}

	#[test]
	fn don_set_target_on_none() {
		let target = None;
		let original_target = Some(Vec3::Y);
		let mut movement = SimpleMovement {
			target: original_target,
			speed: UnitsPerSecond::new(0.),
		};

		movement.set_target(target);

		assert_eq!(original_target, movement.target);
	}
}
