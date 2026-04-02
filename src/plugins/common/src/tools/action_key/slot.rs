use super::{ActionKey, user_input::UserInput};
use crate::{
	errors::{ErrorData, Level},
	tools::is_not::IsNot,
	traits::{
		accessors::get::ViewField,
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{Iter, IterFinite},
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{any::type_name, fmt::Display, marker::PhantomData};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct SlotKey(pub u8);

impl ViewField for SlotKey {
	type TValue<'a> = Self;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum HandSlot {
	#[default]
	Left,
	Right,
}

impl From<HandSlot> for UserInput {
	fn from(value: HandSlot) -> Self {
		match value {
			HandSlot::Left => Self::from(KeyCode::Digit1),
			HandSlot::Right => Self::from(KeyCode::Digit2),
		}
	}
}

impl From<HandSlot> for ActionKey {
	fn from(key: HandSlot) -> Self {
		Self::Slot(key)
	}
}

impl TryFrom<ActionKey> for HandSlot {
	type Error = IsNot<HandSlot>;

	fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
		match key {
			ActionKey::Slot(key) => Ok(key),
			_ => Err(IsNot::target_type()),
		}
	}
}

impl From<HandSlot> for Token {
	fn from(value: HandSlot) -> Self {
		match value {
			HandSlot::Left => Token::from("slot-key-hand-left"),
			HandSlot::Right => Token::from("slot-key-hand-right"),
		}
	}
}

impl IterFinite for HandSlot {
	fn iterator() -> Iter<Self> {
		Iter(Some(HandSlot::Left))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			HandSlot::Left => Some(HandSlot::Right),
			HandSlot::Right => None,
		}
	}
}

impl InvalidUserInput for HandSlot {
	fn invalid_input(&self) -> &[UserInput] {
		&[]
	}
}

impl From<HandSlot> for SlotKey {
	fn from(slot: HandSlot) -> Self {
		Self(match slot {
			HandSlot::Left => 0,
			HandSlot::Right => 1,
		})
	}
}

impl TryFrom<SlotKey> for HandSlot {
	type Error = NoValidAgentKey<HandSlot>;

	fn try_from(slot_key: SlotKey) -> Result<Self, Self::Error> {
		match slot_key {
			SlotKey(0) => Ok(HandSlot::Left),
			SlotKey(1) => Ok(HandSlot::Right),
			slot_key => Err(NoValidAgentKey::for_key(slot_key)),
		}
	}
}

impl PartialEq<HandSlot> for SlotKey {
	fn eq(&self, other: &HandSlot) -> bool {
		self == &SlotKey::from(*other)
	}
}

impl PartialEq<SlotKey> for HandSlot {
	fn eq(&self, other: &SlotKey) -> bool {
		&SlotKey::from(*self) == other
	}
}

impl ViewField for HandSlot {
	type TValue<'a> = Self;
}

#[derive(Debug, PartialEq)]
pub struct NoValidAgentKey<TAgentKey> {
	slot_key: SlotKey,
	_p: PhantomData<TAgentKey>,
}

impl<TAgentKey> NoValidAgentKey<TAgentKey> {
	fn for_key(slot_key: SlotKey) -> Self {
		Self {
			slot_key,
			_p: PhantomData,
		}
	}
}

impl<T> Display for NoValidAgentKey<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let key_name = type_name::<T>();
		let slot_key = self.slot_key;
		write!(f, "The index {slot_key:?} is no valid {key_name}")
	}
}

impl<T> ErrorData for NoValidAgentKey<T> {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"No valid agent key"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod test_hand_slot {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![HandSlot::Left, HandSlot::Right],
			HandSlot::iterator().collect::<Vec<_>>()
		);
	}

	#[test]
	fn hand_key_to_slot_key() {
		assert_eq!(
			vec![SlotKey(0), SlotKey(1)],
			HandSlot::iterator().map(SlotKey::from).collect::<Vec<_>>()
		);
	}

	#[test]
	fn slot_key_to_hand_key() {
		assert_eq!(
			vec![
				Ok(HandSlot::Left),
				Ok(HandSlot::Right),
				Err(NoValidAgentKey::for_key(SlotKey(2))),
			],
			[SlotKey(0), SlotKey(1), SlotKey(2)]
				.into_iter()
				.map(HandSlot::try_from)
				.collect::<Vec<_>>(),
		);
	}
}
