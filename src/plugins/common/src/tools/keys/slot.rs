use super::{IsNot, Key};
use crate::traits::{
	handles_localization::Token,
	iteration::{Iter, IterFinite},
};
use bevy::prelude::*;

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

impl TryFrom<Key> for SlotKey {
	type Error = IsNot<SlotKey>;

	fn try_from(key: Key) -> Result<Self, Self::Error> {
		match key {
			Key::Slot(key) => Ok(key),
			_ => Err(IsNot::key()),
		}
	}
}

impl From<SlotKey> for Token {
	fn from(value: SlotKey) -> Self {
		match value {
			SlotKey::TopHand(Side::Left) => Token::from("slot-key-top-hand-left"),
			SlotKey::TopHand(Side::Right) => Token::from("slot-key-top-hand-right"),
			SlotKey::BottomHand(Side::Left) => Token::from("slot-key-btm-hand-left"),
			SlotKey::BottomHand(Side::Right) => Token::from("slot-key-btm-hand-right"),
		}
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Side {
	Right,
	Left,
}

pub type Combo<TSkill> = Vec<(Vec<SlotKey>, TSkill)>;

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
