pub mod camera_key;
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
use camera_key::CameraKey;
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
	Camera(CameraKey),
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
			Key::Slot(key) => try_next(Key::Slot, key).or(try_fst(Key::Menu)),
			Key::Menu(key) => try_next(Key::Menu, key).or(try_fst(Key::Camera)),
			Key::Camera(key) => try_next(Key::Camera, key),
		}
	}
}

impl From<Key> for UserInput {
	fn from(key: Key) -> Self {
		match key {
			Key::Movement(key) => Self::from(key),
			Key::Slot(key) => Self::from(key),
			Key::Menu(key) => Self::from(key),
			Key::Camera(key) => Self::from(key),
		}
	}
}

impl From<Key> for Token {
	fn from(value: Key) -> Self {
		match value {
			Key::Movement(key) => Self::from(key),
			Key::Slot(key) => Self::from(key),
			Key::Menu(key) => Self::from(key),
			Key::Camera(key) => Self::from(key),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IsNot<TKey>(PhantomData<TKey>);

impl<TKey> IsNot<TKey> {
	pub fn key() -> Self {
		Self(PhantomData)
	}
}

fn try_fst<TInner>(wrap: impl Fn(TInner) -> Key) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

fn try_next<TInner>(wrap: impl Fn(TInner) -> Key, key: TInner) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::next(&Iter(Some(key))).map(wrap)
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
				.chain(MenuState::iterator().map(Key::Menu))
				.chain(CameraKey::iterator().map(Key::Camera))
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
				.chain(MenuState::iterator().map(UserInput::from))
				.chain(CameraKey::iterator().map(UserInput::from))
				.collect::<HashSet<_>>(),
			Key::iterator().map(UserInput::from).collect::<HashSet<_>>(),
		);
	}
}
