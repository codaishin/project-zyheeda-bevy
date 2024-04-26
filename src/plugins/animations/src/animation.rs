use bevy::utils::Uuid;
use common::traits::load_asset::Path;

#[derive(Debug, PartialEq)]
pub enum PlayMode {
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq)]
pub struct Animation {
	uuid: Uuid,
	path: Path,
	play_mode: PlayMode,
}

impl Animation {
	pub fn new_unique(path: Path, play_mode: PlayMode) -> Self {
		Self {
			uuid: Uuid::new_v4(),
			path,
			play_mode,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let path = Path::from("a/path");
		let animation = Animation::new_unique(path.clone(), PlayMode::Repeat);

		assert_eq!(
			(path, PlayMode::Repeat),
			(animation.path, animation.play_mode)
		)
	}

	#[test]
	fn two_animations_with_same_values_are_not_equal() {
		let a = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);
		let b = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);

		assert_ne!(a, b);
	}

	#[test]
	fn animation_is_equal_to_itself() {
		let animation = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);

		assert_eq!(animation, animation);
	}
}
