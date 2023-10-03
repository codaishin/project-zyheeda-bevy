use crate::components::Behaviors;

use super::Empty;

impl Empty for Behaviors {
	fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use crate::behaviors::SimpleMovement;

	use super::*;

	#[test]
	fn is_empty_true() {
		let behaviors = Behaviors(vec![]);

		assert!(behaviors.is_empty());
	}

	#[test]
	fn is_empty_false() {
		let behaviors = Behaviors(vec![SimpleMovement { target: None }]);

		assert!(!behaviors.is_empty());
	}
}
