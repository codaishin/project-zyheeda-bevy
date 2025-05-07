pub mod movement;
pub mod slot;
pub mod user_input;

use crate::{
	states::menu_state::MenuState,
	traits::{
		handles_localization::Token,
		iteration::{Iter, IterFinite},
	},
};
use bevy::{reflect::TypePath, utils::default};
use movement::MovementKey;
use serde::{Deserialize, Serialize};
use slot::SlotKey;
use std::marker::PhantomData;
use user_input::UserInput;

#[derive(TypePath, Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum Key {
	Movement(MovementKey),
	Slot(SlotKey),
	Menu(MenuState),
}

impl Default for Key {
	fn default() -> Self {
		Self::Movement(default())
	}
}

impl IterFinite for Key {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::default()))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Key::Movement(key) => try_next(Key::Movement, key).or(try_fst(Key::Slot)),
			Key::Slot(key) => try_next(Key::Slot, key),
			Key::Menu(_) => None,
		}
	}
}

impl From<Key> for UserInput {
	fn from(key: Key) -> Self {
		match key {
			Key::Movement(key) => Self::from(key),
			Key::Slot(key) => Self::from(key),
			Key::Menu(key) => Self::from(key),
		}
	}
}

impl From<Key> for Token {
	fn from(value: Key) -> Self {
		match value {
			Key::Movement(key) => Self::from(key),
			Key::Slot(key) => Self::from(key),
			Key::Menu(key) => Self::from(key),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IsNot<TKey>(PhantomData<TKey>);

impl<TKey> IsNot<TKey> {
	fn key() -> Self {
		Self(PhantomData)
	}
}

fn try_next<TInner>(wrap: impl Fn(TInner) -> Key, key: TInner) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::next(&Iter(Some(key))).map(wrap)
}

fn try_fst<TInner>(wrap: impl Fn(TInner) -> Key) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::IterFinite;
	use std::collections::HashSet;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			MovementKey::iterator()
				.map(Key::Movement)
				.chain(SlotKey::iterator().map(Key::Slot))
				.collect::<Vec<_>>(),
			Key::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn map_keys() {
		assert_eq!(
			MovementKey::iterator()
				.map(UserInput::from)
				.chain(SlotKey::iterator().map(UserInput::from))
				.collect::<HashSet<_>>(),
			Key::iterator().map(UserInput::from).collect::<HashSet<_>>(),
		);
	}
}
