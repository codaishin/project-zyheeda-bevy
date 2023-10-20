use super::Speed;
use crate::{
	behavior::MovementMode,
	components::{Player, UnitsPerSecond},
};

impl Speed for Player {
	fn get_speed(&self) -> UnitsPerSecond {
		match self.movement_mode {
			MovementMode::Walk => self.walk_speed,
			MovementMode::Run => self.run_speed,
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

		assert_eq!(UnitsPerSecond::new(6.), player.get_speed())
	}

	#[test]
	fn get_run_speed() {
		let player = Player {
			run_speed: UnitsPerSecond::new(60.),
			movement_mode: MovementMode::Run,
			..default()
		};

		assert_eq!(UnitsPerSecond::new(60.), player.get_speed())
	}
}
