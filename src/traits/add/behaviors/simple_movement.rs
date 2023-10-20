use crate::{
	behavior::{BehaviorOld, MovementMode, SimpleMovement},
	components::Behaviors,
	traits::add::Add,
};

impl Add<SimpleMovement> for Behaviors {
	fn add(&mut self, value: SimpleMovement) {
		let walk_movement = BehaviorOld::SimpleMovement((value, MovementMode::Walk));
		self.0.push(walk_movement);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behavior::BehaviorOld, traits::new::New};

	#[test]
	fn add() {
		let mut schedule = Behaviors::new();
		let movement = SimpleMovement { target: None };

		schedule.add(movement);

		assert_eq!(
			vec![BehaviorOld::SimpleMovement((movement, MovementMode::Walk))],
			schedule.0
		);
	}
}
