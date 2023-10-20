use crate::{
	behavior::{BehaviorOld, MovementMode, Run},
	components::Behaviors,
	traits::get::Get,
};

impl Get<Run> for Behaviors {
	fn get(&self) -> Option<Run> {
		match self.0.first() {
			Some(BehaviorOld::SimpleMovement((_, MovementMode::Run))) => Some(Run),
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
	fn get_run() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(BehaviorOld::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Run,
		)));

		let run: Option<Run> = behaviors.get();

		assert_eq!(Some(Run), run);
	}

	#[test]
	fn get_none_when_no_movement() {
		let behaviors = Behaviors::new();

		let run: Option<Run> = behaviors.get();

		assert_eq!(None, run);
	}

	#[test]
	fn get_none_when_not_running() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(BehaviorOld::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Walk,
		)));

		let run: Option<Run> = behaviors.get();

		assert_eq!(None, run);
	}
}
