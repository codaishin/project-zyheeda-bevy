use crate::{
	behavior::{Behavior, MovementMode, SimpleMovement},
	components::Behaviors,
	traits::set::Set,
};

impl Set<SimpleMovement> for Behaviors {
	fn set(&mut self, value: SimpleMovement) {
		let walk_movement = Behavior::SimpleMovement((value, MovementMode::Walk));
		self.0 = vec![walk_movement];
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::Vec3;

	#[test]
	fn set_movement() {
		let movement = SimpleMovement { target: None };
		let mut behaviors = Behaviors(vec![]);

		behaviors.set(movement);

		assert_eq!(
			vec![Behavior::SimpleMovement((movement, MovementMode::Walk))],
			behaviors.0
		);
	}

	#[test]
	fn override_existing() {
		let movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};
		let mut behaviors = Behaviors(vec![
			Behavior::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
			Behavior::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
			Behavior::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
		]);

		behaviors.set(movement);

		assert_eq!(
			vec![Behavior::SimpleMovement((movement, MovementMode::Walk))],
			behaviors.0
		);
	}
}
