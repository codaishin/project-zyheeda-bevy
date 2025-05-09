pub mod camera_key;
pub mod movement;
pub mod slot;
pub mod user_input;

use crate::{
	states::menu_state::MenuState,
	traits::{
		handles_localization::Token,
		handles_settings::InvalidInput,
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
pub enum ActionKey {
	Movement(MovementKey),
	Slot(SlotKey),
	Menu(MenuState),
	Camera(CameraKey),
}

impl Default for ActionKey {
	fn default() -> Self {
		Self::Movement(default())
	}
}

impl From<ActionKey> for UserInput {
	fn from(key: ActionKey) -> Self {
		match key {
			ActionKey::Movement(key) => Self::from(key),
			ActionKey::Slot(key) => Self::from(key),
			ActionKey::Menu(key) => Self::from(key),
			ActionKey::Camera(key) => Self::from(key),
		}
	}
}

impl From<ActionKey> for Token {
	fn from(value: ActionKey) -> Self {
		match value {
			ActionKey::Movement(key) => Self::from(key),
			ActionKey::Slot(key) => Self::from(key),
			ActionKey::Menu(key) => Self::from(key),
			ActionKey::Camera(key) => Self::from(key),
		}
	}
}

impl IterFinite for ActionKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::default()))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			ActionKey::Movement(key) => next(ActionKey::Movement, key).or(first(ActionKey::Slot)),
			ActionKey::Slot(key) => next(ActionKey::Slot, key).or(first(ActionKey::Menu)),
			ActionKey::Menu(key) => next(ActionKey::Menu, key).or(first(ActionKey::Camera)),
			ActionKey::Camera(key) => next(ActionKey::Camera, key),
		}
	}
}

impl InvalidInput<UserInput> for ActionKey {
	fn invalid_input(&self) -> &[UserInput] {
		match self {
			ActionKey::Movement(key) => key.invalid_input(),
			ActionKey::Slot(key) => key.invalid_input(),
			ActionKey::Menu(key) => key.invalid_input(),
			ActionKey::Camera(key) => key.invalid_input(),
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

fn first<TInner>(wrap: impl Fn(TInner) -> ActionKey) -> Option<ActionKey>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

fn next<TInner>(wrap: impl Fn(TInner) -> ActionKey, key: TInner) -> Option<ActionKey>
where
	TInner: IterFinite,
{
	TInner::next(&Iter(Some(key))).map(wrap)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::IterFinite;
	use std::collections::{HashMap, HashSet};

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(ActionKey::Movement))
				.chain(SlotKey::iterator().map(ActionKey::Slot))
				.chain(MenuState::iterator().map(ActionKey::Menu))
				.chain(CameraKey::iterator().map(ActionKey::Camera))
				.collect::<Vec<_>>(),
			ActionKey::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn map_keys() {
		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(UserInput::from))
				.chain(SlotKey::iterator().map(UserInput::from))
				.chain(MenuState::iterator().map(UserInput::from))
				.chain(CameraKey::iterator().map(UserInput::from))
				.collect::<HashSet<_>>(),
			ActionKey::iterator()
				.map(UserInput::from)
				.collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn map_invalid_input() {
		fn pair_with_invalid_input<TKey>(key: TKey) -> (ActionKey, Vec<UserInput>)
		where
			TKey: Into<ActionKey> + InvalidInput<UserInput> + Copy,
		{
			(key.into(), key.invalid_input().to_vec())
		}

		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(pair_with_invalid_input))
				.chain(SlotKey::iterator().map(pair_with_invalid_input))
				.chain(MenuState::iterator().map(pair_with_invalid_input))
				.chain(CameraKey::iterator().map(pair_with_invalid_input))
				.collect::<HashMap<_, _>>(),
			ActionKey::iterator()
				.map(pair_with_invalid_input)
				.collect::<HashMap<_, _>>(),
		);
	}
}
