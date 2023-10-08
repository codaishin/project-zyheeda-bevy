use crate::{
	behavior::{Behavior, SimpleMovement},
	components::Behaviors,
	traits::get::GetMut,
};

impl GetMut<SimpleMovement> for Behaviors {
	fn get(&mut self) -> Option<&mut SimpleMovement> {
		let movement = match self.0.first_mut()? {
			Behavior::SimpleMovement((movement, ..)) => Some(movement),
			_ => None,
		}?;

		_ = movement.target?;

		Some(movement)
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;
	use crate::traits::{add::Add, new::New};

	#[test]
	fn get_none() {
		let mut scheduler = Behaviors::new();

		assert!((&mut scheduler as &mut dyn GetMut<SimpleMovement>)
			.get()
			.is_none());
	}

	#[test]
	fn get_first() {
		let mut scheduler = Behaviors::new();
		let movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};

		scheduler.add(movement);

		assert_eq!(&movement, scheduler.get().unwrap());
	}

	#[test]
	fn get_none_if_target_none() {
		let mut scheduler = Behaviors::new();
		let movement = SimpleMovement { target: None };

		(&mut scheduler as &mut dyn Add<SimpleMovement>).add(movement);

		assert!((&mut scheduler as &mut dyn GetMut<SimpleMovement>)
			.get()
			.is_none());
	}
}
