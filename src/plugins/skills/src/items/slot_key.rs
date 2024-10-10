use bevy::input::keyboard::KeyCode;
use common::{
	components::Side,
	traits::{
		get_ui_text::{English, GetUiText, Japanese, UIText},
		iteration::{Iter, IterFinite},
	},
};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SlotKey {
	Hand(Side),
}

impl Default for SlotKey {
	fn default() -> Self {
		Self::Hand(Side::Right)
	}
}

impl IterFinite for SlotKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(SlotKey::Hand(Side::Right)))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SlotKey::Hand(Side::Right) => Some(SlotKey::Hand(Side::Left)),
			SlotKey::Hand(Side::Left) => None,
		}
	}
}

impl From<SlotKey> for KeyCode {
	fn from(value: SlotKey) -> Self {
		match value {
			SlotKey::Hand(Side::Right) => KeyCode::KeyE,
			SlotKey::Hand(Side::Left) => KeyCode::KeyQ,
		}
	}
}

impl GetUiText<SlotKey> for English {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::Hand(Side::Right) => UIText::from("Main Hand"),
			SlotKey::Hand(Side::Left) => UIText::from("Off Hand"),
		}
	}
}

impl GetUiText<SlotKey> for Japanese {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::Hand(Side::Right) => UIText::from("利き手"),
			SlotKey::Hand(Side::Left) => UIText::from("利き手ではない手"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![SlotKey::Hand(Side::Right), SlotKey::Hand(Side::Left)],
			SlotKey::iterator().collect::<Vec<_>>()
		);
	}
}
