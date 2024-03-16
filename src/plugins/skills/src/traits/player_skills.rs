use crate::{components::Handed, skill::PlayerSkills};
use common::{
	components::{Player, Side},
	traits::{
		iteration::{Iter, IterKey},
		load_asset::Path,
	},
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

impl From<PlayerSkills<Side>> for Path {
	fn from(value: PlayerSkills<Side>) -> Self {
		let value = match value {
			PlayerSkills::Shoot(Handed::Single(Side::Main)) => "Animation4",
			PlayerSkills::Shoot(Handed::Single(Side::Off)) => "Animation5",
			PlayerSkills::Shoot(Handed::Dual(Side::Main)) => "Animation6",
			PlayerSkills::Shoot(Handed::Dual(Side::Off)) => "Animation7",
			PlayerSkills::SwordStrike(Side::Main) => "Animation8",
			PlayerSkills::SwordStrike(Side::Off) => "Animation9",
		};

		Path::from(Player::MODEL_PATH.to_owned() + "#" + value)
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
			.map(Path::from)
			.all(|path| path.as_string().starts_with(&model_path_with_hash)))
	}

	#[test]
	fn no_duplicate_keys() {
		let keys = PlayerSkills::iterator();
		let unique_keys = HashSet::from_iter(PlayerSkills::iterator());
		let unique_strings = HashSet::from_iter(
			PlayerSkills::iterator()
				.map(Path::from)
				.map(|path| path.as_string().clone()),
		);

		assert_eq!(
			(6, 6, 6),
			(keys.count(), unique_keys.len(), unique_strings.len())
		);
	}
}
