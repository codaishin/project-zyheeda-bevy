use super::Clean;
use crate::{
	behavior::{Behavior, Idle},
	components::Behaviors,
};

fn is_active(behavior: &Behavior) -> bool {
	match behavior {
		Behavior::SimpleMovement(movement) => movement.target.is_some(),
		Behavior::Idle(_) => false,
	}
}

impl Clean for Behaviors {
	fn clean(&mut self) {
		let cleaned: Vec<Behavior> = self.0.drain(..).filter(is_active).collect();

		self.0 = if cleaned.is_empty() {
			vec![Behavior::Idle(Idle)]
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
		behavior::{Idle, SimpleMovement},
		traits::{add::Add, new::New, set::Set},
	};

	#[test]
	fn clean_simple_movement() {
		let mut behavior = Behaviors::new();
		behavior.add(SimpleMovement { target: None });

		behavior.clean();

		assert_eq!(vec![Behavior::Idle(Idle)], behavior.0);
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
			vec![Behavior::SimpleMovement(SimpleMovement {
				target: Some(Vec3::ZERO)
			})],
			behavior.0
		);
	}

	#[test]
	fn do_not_clean_idle() {
		let mut behaviors = Behaviors::new();
		behaviors.set(Idle);

		behaviors.clean();

		assert_eq!(vec![Behavior::Idle(Idle)], behaviors.0);
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

		assert_eq!(vec![Behavior::SimpleMovement(movement)], behaviors.0);
	}
}
