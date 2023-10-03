use crate::{behavior::Behavior, components::Behaviors};

use super::Clean;

impl Clean for Behaviors {
	fn clean(&mut self) {
		self.0 = self
			.0
			.drain(..)
			.filter(|behavior| match behavior {
				Behavior::SimpleMovement(movement) => movement.target.is_some(),
				Behavior::Idle(_) => true,
			})
			.collect();
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

		assert!(behavior.0.is_empty());
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
}
