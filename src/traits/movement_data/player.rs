use super::MovementData;
use crate::{
	behaviors::MovementMode,
	components::{Player, UnitsPerSecond},
};

impl MovementData for Player {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
		match self.movement_mode {
			MovementMode::Walk => (self.walk_speed, MovementMode::Walk),
			MovementMode::Run => (self.run_speed, MovementMode::Run),
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
			movement_mode: MovementMode::Walk,
			..default()
		};

		assert_eq!(
			(UnitsPerSecond::new(6.), MovementMode::Walk),
			player.get_movement_data()
		)
	}

	#[test]
	fn get_run_speed() {
		let player = Player {
			run_speed: UnitsPerSecond::new(60.),
			movement_mode: MovementMode::Run,
			..default()
		};

		assert_eq!(
			(UnitsPerSecond::new(60.), MovementMode::Run),
			player.get_movement_data()
		)
	}
}
