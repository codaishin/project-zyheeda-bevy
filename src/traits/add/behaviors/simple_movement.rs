use crate::{behaviors::SimpleMovement, components::Behaviors, traits::add::Add};

impl Add<SimpleMovement> for Behaviors {
	fn add(&mut self, value: SimpleMovement) {
		self.0.push(value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::new::New;

	#[test]
	fn add() {
		let mut schedule = Behaviors::new();
		let movement = SimpleMovement { target: None };
		schedule.add(movement);

		assert_eq!(&movement, schedule.0.first().unwrap());
	}
}
