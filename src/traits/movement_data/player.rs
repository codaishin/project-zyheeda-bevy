use super::MovementData;
use crate::{
	behaviors::MovementMode,
	components::{Animate, Player, PlayerMovement, UnitsPerSecond},
};

impl MovementData<PlayerMovement> for Player {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<PlayerMovement>) {
		match self.movement_mode {
			MovementMode::Slow => (self.walk_speed, Animate::Repeat(PlayerMovement::Walk)),
			MovementMode::Fast => (self.run_speed, Animate::Repeat(PlayerMovement::Run)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn get_walk_speed() {
		let player = Player {
			walk_speed: UnitsPerSecond::new(6.),
			movement_mode: MovementMode::Slow,
			..default()
		};

		assert_eq!(
			(
				UnitsPerSecond::new(6.),
				Animate::Repeat(PlayerMovement::Walk)
			),
			player.get_movement_data()
		)
	}

	#[test]
	fn get_run_speed() {
		let player = Player {
			run_speed: UnitsPerSecond::new(60.),
			movement_mode: MovementMode::Fast,
			..default()
		};

		assert_eq!(
			(
				UnitsPerSecond::new(60.),
				Animate::Repeat(PlayerMovement::Run)
			),
			player.get_movement_data()
		)
	}
}
