use crate::components::BehaviorSchedule;

use super::Clean;

impl Clean for BehaviorSchedule {
	fn clean(&mut self) {
		self.0 = self.0.drain(..).filter(|m| m.target.is_some()).collect();
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;
	use crate::{
		behaviors::SimpleMovement,
		traits::{add::Add, new::New},
	};

	#[test]
	fn clean_simple_movement() {
		let mut state = BehaviorSchedule::new();
		state.add(SimpleMovement { target: None });

		state.clean();

		assert!(state.0.is_empty());
	}

	#[test]
	fn clean_only_when_no_target() {
		let mut state = BehaviorSchedule::new();
		state.add(SimpleMovement { target: None });
		state.add(SimpleMovement {
			target: Some(Vec3::ZERO),
		});

		state.clean();

		assert_eq!(
			(
				1,
				SimpleMovement {
					target: Some(Vec3::ZERO),
				}
			),
			(state.0.len(), state.0[0])
		)
	}
}
