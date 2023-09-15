use crate::{
	components::{BehaviorSchedule, SimpleMovement},
	traits::get::Get,
};

impl Get<SimpleMovement> for BehaviorSchedule {
	fn get(&mut self) -> Option<&mut SimpleMovement> {
		self.0.first_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::UnitsPerSecond,
		traits::{add::Add, new::New},
	};

	#[test]
	fn get_none() {
		let mut scheduler = BehaviorSchedule::new();

		assert!(scheduler.get().is_none());
	}

	#[test]
	fn get_first() {
		let mut scheduler = BehaviorSchedule::new();
		let movement = SimpleMovement {
			speed: UnitsPerSecond::new(0.3),
		};

		scheduler.add(movement);

		assert_eq!(&movement, scheduler.get().unwrap());
	}
}
