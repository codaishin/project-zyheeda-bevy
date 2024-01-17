use super::{Iter, ResourceKey};
use crate::components::PlayerMovement;

const BASE_PATH: &str = "models/player.gltf#";

impl ResourceKey for PlayerMovement {
	fn resource_keys() -> Iter<Self> {
		Iter(Some(PlayerMovement::Walk))
	}

	fn get_next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Self::Walk => Some(PlayerMovement::Run),
			Self::Run => None,
		}
	}

	fn get_resource_path(value: &Self) -> String {
		let value = match value {
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
		assert!(PlayerMovement::resource_keys().all(|(_, path)| path.starts_with(BASE_PATH)))
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerMovement::resource_keys();
		let unique_keys = HashSet::from_iter(PlayerMovement::resource_keys().map(|(key, _)| key));
		let unique_strings =
			HashSet::from_iter(PlayerMovement::resource_keys().map(|(_, str)| str));

		assert_eq!(
			(2, 2, 2),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
