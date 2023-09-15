use crate::{
	components::{BehaviorSchedule, SimpleMovement},
	traits::add::Add,
};

impl Add<SimpleMovement> for BehaviorSchedule {
	fn add(&mut self, value: SimpleMovement) {
		self.0.push(value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::UnitsPerSecond;
	use crate::traits::new::New;

	#[test]
	fn add() {
		let mut schedule = BehaviorSchedule::new();
		let movement = SimpleMovement {
			speed: UnitsPerSecond::new(4.),
		};
		schedule.add(movement);

		assert_eq!(&movement, schedule.0.first().unwrap());
	}
}
