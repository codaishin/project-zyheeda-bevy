use crate::{
	behavior::{BehaviorOld, MovementMode, SimpleMovement},
	components::Behaviors,
	traits::set::Set,
};

impl Set<SimpleMovement> for Behaviors {
	fn set(&mut self, value: SimpleMovement) {
		let walk_movement = BehaviorOld::SimpleMovement((value, MovementMode::Walk));
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
			vec![BehaviorOld::SimpleMovement((movement, MovementMode::Walk))],
			behaviors.0
		);
	}

	#[test]
	fn override_existing() {
		let movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};
		let mut behaviors = Behaviors(vec![
			BehaviorOld::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
			BehaviorOld::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
			BehaviorOld::SimpleMovement((SimpleMovement { target: None }, MovementMode::Walk)),
		]);

		behaviors.set(movement);

		assert_eq!(
			vec![BehaviorOld::SimpleMovement((movement, MovementMode::Walk))],
			behaviors.0
		);
	}
}
