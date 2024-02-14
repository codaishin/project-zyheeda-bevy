use super::MovementData;
use crate::components::{MovementConfig, MovementMode};
use common::tools::UnitsPerSecond;

impl MovementData for MovementConfig {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
		match self {
			MovementConfig::Constant { mode, speed } => (*speed, *mode),
			MovementConfig::Dynamic {
				current_mode: MovementMode::Fast,
				fast_speed,
				..
			} => (*fast_speed, MovementMode::Fast),
			MovementConfig::Dynamic {
				current_mode: MovementMode::Slow,
				slow_speed,
				..
			} => (*slow_speed, MovementMode::Slow),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_constant() {
		let config = MovementConfig::Constant {
			mode: MovementMode::Fast,
			speed: UnitsPerSecond::new(42.),
		};

		assert_eq!(
			(UnitsPerSecond::new(42.), MovementMode::Fast),
			config.get_movement_data()
		);
	}

	#[test]
	fn get_dynamic_fast() {
		let config = MovementConfig::Dynamic {
			current_mode: MovementMode::Fast,
			fast_speed: UnitsPerSecond::new(42.),
			slow_speed: UnitsPerSecond::new(0.),
		};

		assert_eq!(
			(UnitsPerSecond::new(42.), MovementMode::Fast),
			config.get_movement_data()
		);
	}

	#[test]
	fn get_dynamic_slow() {
		let config = MovementConfig::Dynamic {
			current_mode: MovementMode::Slow,
			fast_speed: UnitsPerSecond::new(42.),
			slow_speed: UnitsPerSecond::new(11.),
		};

		assert_eq!(
			(UnitsPerSecond::new(11.), MovementMode::Slow),
			config.get_movement_data()
		);
	}
}
