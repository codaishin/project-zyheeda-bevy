use crate::{
	behavior::{Behavior, SimpleMovement},
	components::Behaviors,
	traits::set::Set,
};

impl Set<SimpleMovement> for Behaviors {
	fn set(&mut self, value: SimpleMovement) {
		self.0 = vec![Behavior::SimpleMovement(value)];
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

		assert_eq!(vec![Behavior::SimpleMovement(movement)], behaviors.0);
	}

	#[test]
	fn override_existing() {
		let movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};
		let mut behaviors = Behaviors(vec![
			Behavior::SimpleMovement(SimpleMovement { target: None }),
			Behavior::SimpleMovement(SimpleMovement { target: None }),
			Behavior::SimpleMovement(SimpleMovement { target: None }),
		]);

		behaviors.set(movement);

		assert_eq!(vec![Behavior::SimpleMovement(movement)], behaviors.0);
	}
}
