use super::Clean;
use crate::{
	behavior::{BehaviorOld, Idle},
	components::Behaviors,
};

fn is_active(behavior: &BehaviorOld) -> bool {
	match behavior {
		BehaviorOld::SimpleMovement((movement, ..)) => movement.target.is_some(),
		BehaviorOld::Idle(_) => false,
	}
}

impl Clean for Behaviors {
	fn clean(&mut self) {
		let cleaned: Vec<BehaviorOld> = self.0.drain(..).filter(is_active).collect();

		self.0 = if cleaned.is_empty() {
			vec![BehaviorOld::Idle(Idle)]
		} else {
			cleaned
		};
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;
	use crate::{
		behavior::{Idle, MovementMode, SimpleMovement},
		traits::{add::Add, new::New, set::Set},
	};

	#[test]
	fn clean_simple_movement() {
		let mut behavior = Behaviors::new();
		behavior.add(SimpleMovement { target: None });

		behavior.clean();

		assert_eq!(vec![BehaviorOld::Idle(Idle)], behavior.0);
	}

	#[test]
	fn clean_only_when_simple_movement_has_no_target() {
		let mut behavior = Behaviors::new();
		behavior.add(SimpleMovement { target: None });
		behavior.add(SimpleMovement {
			target: Some(Vec3::ZERO),
		});

		behavior.clean();

		assert_eq!(
			vec![BehaviorOld::SimpleMovement((
				SimpleMovement {
					target: Some(Vec3::ZERO)
				},
				MovementMode::Walk
			))],
			behavior.0
		);
	}

	#[test]
	fn do_not_clean_idle() {
		let mut behaviors = Behaviors::new();
		behaviors.set(Idle);

		behaviors.clean();

		assert_eq!(vec![BehaviorOld::Idle(Idle)], behaviors.0);
	}

	#[test]
	fn clean_idle_when_any_other_enqueued() {
		let mut behaviors = Behaviors::new();
		let movement = SimpleMovement {
			target: Some(Vec3::default()),
		};
		behaviors.set(Idle);
		behaviors.add(movement);

		behaviors.clean();

		assert_eq!(
			vec![BehaviorOld::SimpleMovement((movement, MovementMode::Walk))],
			behaviors.0
		);
	}
}
