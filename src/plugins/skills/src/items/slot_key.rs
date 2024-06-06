use bevy::input::keyboard::KeyCode;
use common::{
	components::Side,
	traits::{
		get_ui_text::{English, GetUiText, Japanese, UIText},
		iteration::{Iter, IterKey},
	},
};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SlotKey {
	Hand(Side),
}

impl Default for SlotKey {
	fn default() -> Self {
		Self::Hand(Side::Main)
	}
}

impl IterKey for SlotKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(SlotKey::Hand(Side::Main)))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SlotKey::Hand(Side::Main) => Some(SlotKey::Hand(Side::Off)),
			SlotKey::Hand(Side::Off) => None,
		}
	}
}

impl From<SlotKey> for KeyCode {
	fn from(value: SlotKey) -> Self {
		match value {
			SlotKey::Hand(Side::Main) => KeyCode::KeyE,
			SlotKey::Hand(Side::Off) => KeyCode::KeyQ,
		}
	}
}

impl GetUiText<SlotKey> for English {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::Hand(Side::Main) => UIText::from("Main Hand"),
			SlotKey::Hand(Side::Off) => UIText::from("Off Hand"),
		}
	}
}

impl GetUiText<SlotKey> for Japanese {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::Hand(Side::Main) => UIText::from("利き手"),
			SlotKey::Hand(Side::Off) => UIText::from("利き手ではない手"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			SlotKey::iterator().collect::<Vec<_>>()
		);
	}
}
