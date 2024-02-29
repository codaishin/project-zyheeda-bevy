use crate::{components::Handed, skill::PlayerSkills};
use common::{
	components::{Player, Side},
	traits::iteration::{Iter, IterKey, KeyValue},
};

impl IterKey for PlayerSkills<Side> {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Shoot(Handed::Single(Side::Main))))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
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
			Self::Shoot(Handed::Single(Side::Main)) => "Animation4",
			Self::Shoot(Handed::Single(Side::Off)) => "Animation5",
			Self::Shoot(Handed::Dual(Side::Main)) => "Animation6",
			Self::Shoot(Handed::Dual(Side::Off)) => "Animation7",
			Self::SwordStrike(Side::Main) => "Animation8",
			Self::SwordStrike(Side::Off) => "Animation9",
		};

		Player::MODEL_PATH.to_owned() + "#" + value
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::HashSet;

	#[test]
	fn all_contain_base_path() {
		let model_path_with_hash = Player::MODEL_PATH.to_owned() + "#";
		assert!(PlayerSkills::iterator()
			.map(PlayerSkills::<Side>::get_value)
			.all(|path| path.starts_with(&model_path_with_hash)))
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerSkills::iterator();
		let unique_keys = HashSet::from_iter(PlayerSkills::iterator());
		let unique_strings =
			HashSet::from_iter(PlayerSkills::iterator().map(PlayerSkills::<Side>::get_value));

		assert_eq!(
			(6, 6, 6),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
