use crate::{
	behavior::{BehaviorOld, MovementMode, SimpleMovement},
	components::Behaviors,
	traits::get::GetMut,
};

impl GetMut<(SimpleMovement, MovementMode)> for Behaviors {
	fn get(&mut self) -> Option<&mut (SimpleMovement, MovementMode)> {
		let movement = match self.0.first_mut()? {
			BehaviorOld::SimpleMovement(movement) => Some(movement),
			_ => None,
		}?;

		_ = movement.0.target?;

		Some(movement)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::new::New;
	use bevy::prelude::Vec3;

	#[test]
	fn get_none() {
		let mut behaviors = Behaviors::new();

		let movement: Option<&mut (SimpleMovement, MovementMode)> = behaviors.get();

		assert!(movement.is_none());
	}

	#[test]
	fn get_first() {
		let mut behaviors = Behaviors::new();
		let expected = (
			SimpleMovement {
				target: Some(Vec3::ONE),
			},
			MovementMode::Run,
		);

		behaviors.0.push(BehaviorOld::SimpleMovement(expected));

		assert_eq!(&expected, behaviors.get().unwrap());
	}

	#[test]
	fn get_none_if_target_none() {
		let mut behaviors = Behaviors::new();

		behaviors.0.push(BehaviorOld::SimpleMovement((
			SimpleMovement { target: None },
			MovementMode::Run,
		)));

		let movement: Option<&mut (SimpleMovement, MovementMode)> = behaviors.get();

		assert!(movement.is_none());
	}
}
