use crate::{components::Handed, skill::PlayerSkills};
use common::{
	components::Side,
	traits::iteration::{Iter, IterKey, KeyValue},
};

const BASE_PATH: &str = "models/player.gltf#";

impl IterKey for PlayerSkills<Side> {
	fn iterator() -> Iter<Self> {
		Iter(Some(PlayerSkills::Idle))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Self::Idle => Some(Self::Shoot(Handed::Single(Side::Main))),
			Self::Shoot(Handed::Single(Side::Main)) => Some(Self::Shoot(Handed::Single(Side::Off))),
			Self::Shoot(Handed::Single(Side::Off)) => Some(Self::Shoot(Handed::Dual(Side::Main))),
			Self::Shoot(Handed::Dual(Side::Main)) => Some(Self::Shoot(Handed::Dual(Side::Off))),
			Self::Shoot(Handed::Dual(Side::Off)) => Some(Self::SwordStrike(Side::Main)),
			Self::SwordStrike(Side::Main) => Some(Self::SwordStrike(Side::Off)),
			Self::SwordStrike(Side::Off) => None,
		}
	}
}

impl KeyValue<String> for PlayerSkills<Side> {
	fn get_value(self) -> String {
		let value = match self {
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
		assert!(PlayerSkills::iterator()
			.map(PlayerSkills::<Side>::get_value)
			.all(|path| path.starts_with(BASE_PATH)))
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerSkills::iterator();
		let unique_keys = HashSet::from_iter(PlayerSkills::iterator());
		let unique_strings =
			HashSet::from_iter(PlayerSkills::iterator().map(PlayerSkills::<Side>::get_value));

		assert_eq!(
			(7, 7, 7),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
