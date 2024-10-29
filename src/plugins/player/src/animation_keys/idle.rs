use crate::components::player::Player;
use common::traits::{
	iteration::{Iter, IterFinite},
	load_asset::Path,
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PlayerIdle;

impl IterFinite for PlayerIdle {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self))
	}

	fn next(_: &Iter<Self>) -> Option<Self> {
		None
	}
}

impl From<PlayerIdle> for Path {
	fn from(_: PlayerIdle) -> Self {
		Player::animation_path("Animation2")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_only_default() {
		let iterations: Vec<_> = PlayerIdle::iterator().collect();

		assert_eq!(vec![PlayerIdle], iterations);
	}

	#[test]
	fn values() {
		let iterations: Vec<_> = PlayerIdle::iterator()
			.map(Path::from)
			.map(|path| path.as_string().clone())
			.collect();

		assert_eq!(
			vec![Player::MODEL_PATH.to_owned() + "#Animation2"],
			iterations
		);
	}
}
