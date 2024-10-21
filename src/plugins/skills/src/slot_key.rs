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
	TopHand(Side),
	BottomHand(Side),
}

impl Default for SlotKey {
	fn default() -> Self {
		Self::BottomHand(Side::Right)
	}
}

impl IterFinite for SlotKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(SlotKey::TopHand(Side::Left)))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SlotKey::TopHand(Side::Left) => Some(SlotKey::BottomHand(Side::Left)),
			SlotKey::BottomHand(Side::Left) => Some(SlotKey::BottomHand(Side::Right)),
			SlotKey::BottomHand(Side::Right) => Some(SlotKey::TopHand(Side::Right)),
			SlotKey::TopHand(Side::Right) => None,
		}
	}
}

impl From<SlotKey> for KeyCode {
	fn from(value: SlotKey) -> Self {
		match value {
			SlotKey::TopHand(Side::Left) => KeyCode::Digit1,
			SlotKey::BottomHand(Side::Left) => KeyCode::Digit2,
			SlotKey::BottomHand(Side::Right) => KeyCode::Digit3,
			SlotKey::TopHand(Side::Right) => KeyCode::Digit4,
		}
	}
}

impl GetUiText<SlotKey> for English {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::TopHand(Side::Right) => UIText::from("Right Hand (Top)"),
			SlotKey::TopHand(Side::Left) => UIText::from("Left Hand (Top)"),
			SlotKey::BottomHand(Side::Right) => UIText::from("Right Hand (Bottom)"),
			SlotKey::BottomHand(Side::Left) => UIText::from("Left Hand (Bottom)"),
		}
	}
}

impl GetUiText<SlotKey> for Japanese {
	fn ui_text(value: &SlotKey) -> UIText {
		match value {
			SlotKey::TopHand(Side::Right) => UIText::from("右手「上」"),
			SlotKey::TopHand(Side::Left) => UIText::from("左手「上」"),
			SlotKey::BottomHand(Side::Right) => UIText::from("右手「下」"),
			SlotKey::BottomHand(Side::Left) => UIText::from("左手「下」"),
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
				SlotKey::TopHand(Side::Left),
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right),
				SlotKey::TopHand(Side::Right),
			],
			SlotKey::iterator().collect::<Vec<_>>()
		);
	}
}
