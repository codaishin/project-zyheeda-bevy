use crate::components::PlayerMovement;
use behaviors::components::{MovementConfig, MovementMode};
use common::{
	components::Player,
	traits::{
		iteration::{Iter, IterKey},
		load_asset::Path,
	},
};

impl IterKey for PlayerMovement {
	fn iterator() -> Iter<Self> {
		Iter(Some(PlayerMovement::Walk))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Self::Walk => Some(PlayerMovement::Run),
			Self::Run => None,
		}
	}
}

impl From<PlayerMovement> for Path {
	fn from(value: PlayerMovement) -> Self {
		let value = match value {
			PlayerMovement::Walk => "Animation1",
			PlayerMovement::Run => "Animation3",
		};

		Path::from(Player::MODEL_PATH.to_owned() + "#" + value)
	}
}

impl From<MovementConfig> for PlayerMovement {
	fn from(config: MovementConfig) -> Self {
		if is_fast(config) {
			PlayerMovement::Run
		} else {
			PlayerMovement::Walk
		}
	}
}

fn is_fast(config: MovementConfig) -> bool {
	matches!(
		config,
		MovementConfig::Constant {
			mode: MovementMode::Fast,
			..
		} | MovementConfig::Dynamic {
			current_mode: MovementMode::Fast,
			..
		}
	)
}

#[cfg(test)]
mod test_iteration {
	use super::*;
	use bevy::utils::HashSet;

	#[test]
	fn all_contain_base_path() {
		let model_path_with_hash = Player::MODEL_PATH.to_owned() + "#";
		assert!(PlayerMovement::iterator()
			.map(Path::from)
			.all(|path| path.as_string().starts_with(&model_path_with_hash)));
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerMovement::iterator();
		let unique_keys = HashSet::from_iter(PlayerMovement::iterator());
		let unique_strings = HashSet::from_iter(
			PlayerMovement::iterator()
				.map(Path::from)
				.map(|p| p.as_string().clone()),
		);

		assert_eq!(
			(2, 2, 2),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}

#[cfg(test)]
mod test_from_movement_mode {
	use super::*;
	use common::tools::UnitsPerSecond;

	#[test]
	fn constant_fast_to_run() {
		let mode = PlayerMovement::from(MovementConfig::Constant {
			mode: MovementMode::Fast,
			speed: UnitsPerSecond::default(),
		});
		assert_eq!(PlayerMovement::Run, mode);
	}

	#[test]
	fn constant_slow_to_walk() {
		let mode = PlayerMovement::from(MovementConfig::Constant {
			mode: MovementMode::Slow,
			speed: UnitsPerSecond::default(),
		});
		assert_eq!(PlayerMovement::Walk, mode);
	}

	#[test]
	fn dynamic_fast_to_run() {
		let mode = PlayerMovement::from(MovementConfig::Dynamic {
			current_mode: MovementMode::Fast,
			slow_speed: UnitsPerSecond::default(),
			fast_speed: UnitsPerSecond::default(),
		});
		assert_eq!(PlayerMovement::Run, mode);
	}

	#[test]
	fn dynamic_slow_to_walk() {
		let mode = PlayerMovement::from(MovementConfig::Dynamic {
			current_mode: MovementMode::Slow,
			slow_speed: UnitsPerSecond::default(),
			fast_speed: UnitsPerSecond::default(),
		});
		assert_eq!(PlayerMovement::Walk, mode);
	}
}
