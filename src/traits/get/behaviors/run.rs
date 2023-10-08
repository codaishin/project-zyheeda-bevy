use crate::{
	behavior::{Behavior, MovementMode, Run},
	components::Behaviors,
	traits::get::Get,
};

impl Get<Run> for Behaviors {
	fn get(&mut self) -> Option<Run> {
		match self.0.first() {
			Some(Behavior::SimpleMovement((_, MovementMode::Run))) => Some(Run),
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
	fn get_run() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(Behavior::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Run,
		)));

		let run: Option<Run> = behaviors.get();

		assert_eq!(Some(Run), run);
	}

	#[test]
	fn get_none_when_no_movement() {
		let mut behaviors = Behaviors::new();

		let run: Option<Run> = behaviors.get();

		assert_eq!(None, run);
	}

	#[test]
	fn get_none_when_not_running() {
		let mut behaviors = Behaviors::new();
		behaviors.0.push(Behavior::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Walk,
		)));

		let run: Option<Run> = behaviors.get();

		assert_eq!(None, run);
	}
}
