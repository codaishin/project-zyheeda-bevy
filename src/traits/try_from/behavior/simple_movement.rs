use crate::{behaviors::Behavior, components::SimpleMovement};

impl TryFrom<Behavior> for SimpleMovement {
	type Error = ();

	fn try_from(behavior: Behavior) -> Result<Self, Self::Error> {
		match behavior {
			Behavior::MoveTo(target) => Ok(SimpleMovement { target }),
		}
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::Vec3;

	use super::*;

	#[test]
	fn get_simple_movement() {
		let behavior = Behavior::MoveTo(Vec3::ONE);
		let movement: Result<SimpleMovement, ()> = behavior.try_into();

		assert_eq!(Ok(Vec3::ONE), movement.map(|m| m.target));
	}
}
