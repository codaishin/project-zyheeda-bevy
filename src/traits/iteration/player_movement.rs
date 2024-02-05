use super::{Iter, IterKey, KeyValue};
use crate::components::PlayerMovement;

const BASE_PATH: &str = "models/player.gltf#";

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

impl KeyValue<String> for PlayerMovement {
	fn get_value(self) -> String {
		let value = match self {
			Self::Walk => "Animation1",
			Self::Run => "Animation3",
		};

		BASE_PATH.to_owned() + value
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::HashSet;

	#[test]
	fn all_contain_base_path() {
		assert!(PlayerMovement::iterator()
			.map(PlayerMovement::get_value)
			.all(|path| path.starts_with(BASE_PATH)));
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerMovement::iterator();
		let unique_keys = HashSet::from_iter(PlayerMovement::iterator());
		let unique_strings =
			HashSet::from_iter(PlayerMovement::iterator().map(PlayerMovement::get_value));

		assert_eq!(
			(2, 2, 2),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
