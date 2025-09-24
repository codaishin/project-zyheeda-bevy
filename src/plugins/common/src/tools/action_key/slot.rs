use std::any::type_name;

use super::{ActionKey, user_input::UserInput};
use crate::{
	errors::{Error, IsNot, Level},
	traits::{
		accessors::get::Property,
		handles_localization::Token,
		handles_settings::InvalidInput,
		iteration::{Iter, IterFinite},
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct SlotKey(pub u8);

impl Property for SlotKey {
	type TValue<'a> = Self;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum PlayerSlot {
	Upper(Side),
	Lower(Side),
}

impl PlayerSlot {
	pub const UPPER_L: Self = Self::Upper(Side::Left);
	pub const UPPER_R: Self = Self::Upper(Side::Right);
	pub const LOWER_L: Self = Self::Lower(Side::Left);
	pub const LOWER_R: Self = Self::Lower(Side::Right);
}

impl Default for PlayerSlot {
	fn default() -> Self {
		Self::Lower(Side::Right)
	}
}

impl From<PlayerSlot> for UserInput {
	fn from(value: PlayerSlot) -> Self {
		match value {
			PlayerSlot::Upper(Side::Left) => Self::from(KeyCode::Digit1),
			PlayerSlot::Lower(Side::Left) => Self::from(KeyCode::Digit2),
			PlayerSlot::Lower(Side::Right) => Self::from(KeyCode::Digit3),
			PlayerSlot::Upper(Side::Right) => Self::from(KeyCode::Digit4),
		}
	}
}

impl From<PlayerSlot> for ActionKey {
	fn from(key: PlayerSlot) -> Self {
		Self::Slot(key)
	}
}

impl TryFrom<ActionKey> for PlayerSlot {
	type Error = IsNot<PlayerSlot>;

	fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
		match key {
			ActionKey::Slot(key) => Ok(key),
			_ => Err(IsNot::target_type()),
		}
	}
}

impl From<PlayerSlot> for Token {
	fn from(value: PlayerSlot) -> Self {
		match value {
			PlayerSlot::Upper(Side::Left) => Token::from("slot-key-top-hand-left"),
			PlayerSlot::Upper(Side::Right) => Token::from("slot-key-top-hand-right"),
			PlayerSlot::Lower(Side::Left) => Token::from("slot-key-btm-hand-left"),
			PlayerSlot::Lower(Side::Right) => Token::from("slot-key-btm-hand-right"),
		}
	}
}

impl IterFinite for PlayerSlot {
	fn iterator() -> Iter<Self> {
		Iter(Some(PlayerSlot::Upper(Side::Left)))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			PlayerSlot::Upper(Side::Left) => Some(PlayerSlot::Lower(Side::Left)),
			PlayerSlot::Lower(Side::Left) => Some(PlayerSlot::Lower(Side::Right)),
			PlayerSlot::Lower(Side::Right) => Some(PlayerSlot::Upper(Side::Right)),
			PlayerSlot::Upper(Side::Right) => None,
		}
	}
}

impl InvalidInput for PlayerSlot {
	type TInput = UserInput;

	fn invalid_input(&self) -> &[UserInput] {
		&[]
	}
}

impl From<PlayerSlot> for SlotKey {
	fn from(slot: PlayerSlot) -> Self {
		Self(match slot {
			PlayerSlot::Upper(Side::Left) => 0,
			PlayerSlot::Lower(Side::Left) => 1,
			PlayerSlot::Lower(Side::Right) => 2,
			PlayerSlot::Upper(Side::Right) => 3,
		})
	}
}

impl TryFrom<SlotKey> for PlayerSlot {
	type Error = NoValidSlotKey;

	fn try_from(slot_key: SlotKey) -> Result<Self, Self::Error> {
		match slot_key {
			SlotKey(0) => Ok(PlayerSlot::Upper(Side::Left)),
			SlotKey(1) => Ok(PlayerSlot::Lower(Side::Left)),
			SlotKey(2) => Ok(PlayerSlot::Lower(Side::Right)),
			SlotKey(3) => Ok(PlayerSlot::Upper(Side::Right)),
			slot_key => Err(NoValidSlotKey { slot_key }),
		}
	}
}

impl Property for PlayerSlot {
	type TValue<'a> = Self;
}

#[derive(Debug, PartialEq)]
pub struct NoValidSlotKey {
	pub slot_key: SlotKey,
}

impl From<NoValidSlotKey> for Error {
	fn from(NoValidSlotKey { slot_key: raw }: NoValidSlotKey) -> Self {
		let key_name = type_name::<PlayerSlot>();

		Self::Single {
			msg: format!("the index {raw:?} is no valid {key_name}"),
			lvl: Level::Error,
		}
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum Side {
	Right,
	Left,
}

#[cfg(test)]
mod test_player_slot {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![
				PlayerSlot::Upper(Side::Left),
				PlayerSlot::Lower(Side::Left),
				PlayerSlot::Lower(Side::Right),
				PlayerSlot::Upper(Side::Right),
			],
			PlayerSlot::iterator().collect::<Vec<_>>()
		);
	}

	#[test]
	fn player_key_to_slot_key() {
		assert_eq!(
			vec![SlotKey(0), SlotKey(1), SlotKey(2), SlotKey(3)],
			PlayerSlot::iterator()
				.map(SlotKey::from)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn slot_key_to_player_key() {
		assert_eq!(
			vec![
				Ok(PlayerSlot::Upper(Side::Left)),
				Ok(PlayerSlot::Lower(Side::Left)),
				Ok(PlayerSlot::Lower(Side::Right)),
				Ok(PlayerSlot::Upper(Side::Right)),
				Err(NoValidSlotKey {
					slot_key: SlotKey(4)
				}),
			],
			[SlotKey(0), SlotKey(1), SlotKey(2), SlotKey(3), SlotKey(4)]
				.into_iter()
				.map(PlayerSlot::try_from)
				.collect::<Vec<_>>(),
		);
	}
}
