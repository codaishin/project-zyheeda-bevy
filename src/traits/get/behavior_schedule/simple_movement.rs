use crate::{
	components::{BehaviorSchedule, SimpleMovement},
	traits::get::Get,
};

impl Get<SimpleMovement> for BehaviorSchedule {
	fn get(&mut self) -> Option<&mut SimpleMovement> {
		let first = self.0.first_mut()?;
		_ = first.target?;

		Some(first)
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;
	use crate::traits::{add::Add, new::New};

	#[test]
	fn get_none() {
		let mut scheduler = BehaviorSchedule::new();

		assert!(scheduler.get().is_none());
	}

	#[test]
	fn get_first() {
		let mut scheduler = BehaviorSchedule::new();
		let movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};

		scheduler.add(movement);

		assert_eq!(&movement, scheduler.get().unwrap());
	}

	#[test]
	fn get_none_if_target_none() {
		let mut scheduler = BehaviorSchedule::new();
		let movement = SimpleMovement { target: None };

		scheduler.add(movement);

		assert!(scheduler.get().is_none());
	}
}
