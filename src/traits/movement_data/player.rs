use super::MovementData;
use crate::{
	behaviors::MovementMode,
	components::{Player, UnitsPerSecond},
};

impl MovementData for Player {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
		match self.movement_mode {
			MovementMode::Slow => (self.walk_speed, MovementMode::Slow),
			MovementMode::Fast => (self.run_speed, MovementMode::Fast),
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
			(UnitsPerSecond::new(6.), MovementMode::Slow),
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
			(UnitsPerSecond::new(60.), MovementMode::Fast),
			player.get_movement_data()
		)
	}
}
