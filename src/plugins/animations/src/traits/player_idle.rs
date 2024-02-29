use crate::animation_keys::PlayerIdle;
use common::{
	components::Player,
	traits::iteration::{Iter, IterKey, KeyValue},
};

impl IterKey for PlayerIdle {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self))
	}

	fn next(_: &Iter<Self>) -> Option<Self> {
		None
	}
}

impl KeyValue<String> for PlayerIdle {
	fn get_value(self) -> String {
		Player::MODEL_PATH.to_owned() + "#Animation2"
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
		let iterations: Vec<_> = PlayerIdle::iterator().map(|i| i.get_value()).collect();

		assert_eq!(
			vec![Player::MODEL_PATH.to_owned() + "#Animation2"],
			iterations
		);
	}
}
