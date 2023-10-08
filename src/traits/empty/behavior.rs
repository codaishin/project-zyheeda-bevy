use crate::components::Behaviors;

use super::Empty;

impl Empty for Behaviors {
	fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behavior::{Behavior, MovementMode, SimpleMovement};

	#[test]
	fn is_empty_true() {
		let behaviors = Behaviors(vec![]);

		assert!(behaviors.is_empty());
	}

	#[test]
	fn is_empty_false() {
		let behaviors = Behaviors(vec![Behavior::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Walk,
		))]);

		assert!(!behaviors.is_empty());
	}
}
