use super::{Iter, ResourceKey};
use crate::components::{Handed, PlayerSkills, Side};

const BASE_PATH: &str = "models/player.gltf#";

impl ResourceKey for PlayerSkills<Side> {
	fn resource_keys() -> Iter<Self> {
		Iter(Some(PlayerSkills::Idle))
	}

	fn get_next(current: &Iter<Self>) -> Option<Self> {
		match &current.0? {
			Self::Idle => Some(Self::Shoot(Handed::Single(Side::Main))),
			Self::Shoot(Handed::Single(Side::Main)) => Some(Self::Shoot(Handed::Single(Side::Off))),
			Self::Shoot(Handed::Single(Side::Off)) => Some(Self::Shoot(Handed::Dual(Side::Main))),
			Self::Shoot(Handed::Dual(Side::Main)) => Some(Self::Shoot(Handed::Dual(Side::Off))),
			Self::Shoot(Handed::Dual(Side::Off)) => Some(Self::SwordStrike(Side::Main)),
			Self::SwordStrike(Side::Main) => Some(Self::SwordStrike(Side::Off)),
			Self::SwordStrike(Side::Off) => None,
		}
	}

	fn get_resource_path(value: &Self) -> String {
		let value = match value {
			Self::Idle => "Animation2",
			Self::Shoot(Handed::Single(Side::Main)) => "Animation4",
			Self::Shoot(Handed::Single(Side::Off)) => "Animation5",
			Self::Shoot(Handed::Dual(Side::Main)) => "Animation6",
			Self::Shoot(Handed::Dual(Side::Off)) => "Animation7",
			Self::SwordStrike(Side::Main) => "Animation8",
			Self::SwordStrike(Side::Off) => "Animation9",
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
		assert!(PlayerSkills::resource_keys().all(|(_, path)| path.starts_with(BASE_PATH)))
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerSkills::resource_keys();
		let unique_keys = HashSet::from_iter(PlayerSkills::resource_keys().map(|(key, _)| key));
		let unique_strings = HashSet::from_iter(PlayerSkills::resource_keys().map(|(_, str)| str));

		assert_eq!(
			(7, 7, 7),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
