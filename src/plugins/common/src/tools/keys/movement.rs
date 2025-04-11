use crate::traits::{
	get_ui_text::{English, GetUiText, Japanese, UIText},
	iteration::{Iter, IterFinite},
};

#[derive(Default, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum MovementKey {
	#[default]
	Forward,
	Backward,
	Left,
	Right,
}

impl IterFinite for MovementKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::default()))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			MovementKey::Forward => Some(MovementKey::Backward),
			MovementKey::Backward => Some(MovementKey::Left),
			MovementKey::Left => Some(MovementKey::Right),
			MovementKey::Right => None,
		}
	}
}

impl GetUiText<MovementKey> for English {
	fn ui_text(key: &MovementKey) -> UIText {
		match key {
			MovementKey::Forward => UIText::from("move forward"),
			MovementKey::Backward => UIText::from("move backward"),
			MovementKey::Left => UIText::from("move left"),
			MovementKey::Right => UIText::from("move right"),
		}
	}
}

impl GetUiText<MovementKey> for Japanese {
	fn ui_text(key: &MovementKey) -> UIText {
		match key {
			MovementKey::Forward => UIText::from("前進"),
			MovementKey::Backward => UIText::from("後退"),
			MovementKey::Left => UIText::from("左に移動"),
			MovementKey::Right => UIText::from("右に移動"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![
				MovementKey::Forward,
				MovementKey::Backward,
				MovementKey::Left,
				MovementKey::Right,
			],
			MovementKey::iterator().take(5).collect::<Vec<_>>()
		);
	}
}
