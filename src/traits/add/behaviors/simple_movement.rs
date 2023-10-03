use crate::{
	behavior::{Behavior, SimpleMovement},
	components::Behaviors,
	traits::add::Add,
};

impl Add<SimpleMovement> for Behaviors {
	fn add(&mut self, value: SimpleMovement) {
		self.0.push(Behavior::SimpleMovement(value));
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

		assert_eq!(vec![Behavior::SimpleMovement(movement)], schedule.0);
	}
}
