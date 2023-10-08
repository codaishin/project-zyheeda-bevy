use crate::{
	behavior::{Behavior, MovementMode, SimpleMovement},
	components::Behaviors,
	traits::add::Add,
};

impl Add<SimpleMovement> for Behaviors {
	fn add(&mut self, value: SimpleMovement) {
		let walk_movement = Behavior::SimpleMovement((value, MovementMode::Walk));
		self.0.push(walk_movement);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behavior::Behavior, traits::new::New};

	#[test]
	fn add() {
		let mut schedule = Behaviors::new();
		let movement = SimpleMovement { target: None };

		schedule.add(movement);

		assert_eq!(
			vec![Behavior::SimpleMovement((movement, MovementMode::Walk))],
			schedule.0
		);
	}
}
