use crate::{
	behavior::{BehaviorOld, MovementMode, Walk},
	components::Behaviors,
	traits::get::Get,
};

impl Get<Walk> for Behaviors {
	fn get(&self) -> Option<Walk> {
		match self.0.first() {
			Some(BehaviorOld::SimpleMovement((_, MovementMode::Walk))) => Some(Walk),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behavior::{BehaviorOld, MovementMode, SimpleMovement},
		traits::new::New,
	};

	#[test]
	fn get_walk() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(BehaviorOld::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Walk,
		)));

		let walk: Option<Walk> = behaviors.get();

		assert_eq!(Some(Walk), walk);
	}

	#[test]
	fn get_none_when_no_movement() {
		let behaviors = Behaviors::new();

		let walk: Option<Walk> = behaviors.get();

		assert_eq!(None, walk);
	}

	#[test]
	fn get_none_when_not_walking() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(BehaviorOld::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Run,
		)));

		let walk: Option<Walk> = behaviors.get();

		assert_eq!(None, walk);
	}
}
