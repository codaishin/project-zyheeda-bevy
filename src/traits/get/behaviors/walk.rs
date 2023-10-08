use crate::{
	behavior::{Behavior, Walk},
	components::Behaviors,
	traits::get::Get,
};

impl Get<Walk> for Behaviors {
	fn get(&mut self) -> Option<Walk> {
		match self.0.first_mut() {
			Some(Behavior::SimpleMovement(_)) => Some(Walk),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behavior::{Behavior, MovementMode, SimpleMovement},
		traits::new::New,
	};

	#[test]
	fn get_walk() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(Behavior::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Walk,
		)));

		let walk: Option<Walk> = behaviors.get();

		assert_eq!(Some(Walk), walk);
	}

	#[test]
	fn get_none_when_no_movement() {
		let mut behaviors = Behaviors::new();

		let walk: Option<Walk> = behaviors.get();

		assert_eq!(None, walk);
	}
}
