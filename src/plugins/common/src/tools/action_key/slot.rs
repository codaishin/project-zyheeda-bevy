use super::{IsNot, ActionKey, user_input::UserInput};
use crate::traits::{
	handles_localization::Token,
	iteration::{Iter, IterFinite},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
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

impl From<SlotKey> for UserInput {
	fn from(value: SlotKey) -> Self {
		match value {
			SlotKey::TopHand(Side::Left) => Self::from(KeyCode::Digit1),
			SlotKey::BottomHand(Side::Left) => Self::from(KeyCode::Digit2),
			SlotKey::BottomHand(Side::Right) => Self::from(KeyCode::Digit3),
			SlotKey::TopHand(Side::Right) => Self::from(KeyCode::Digit4),
		}
	}
}

impl From<SlotKey> for ActionKey {
	fn from(key: SlotKey) -> Self {
		Self::Slot(key)
	}
}

impl TryFrom<ActionKey> for SlotKey {
	type Error = IsNot<SlotKey>;

	fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
		match key {
			ActionKey::Slot(key) => Ok(key),
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

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
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
