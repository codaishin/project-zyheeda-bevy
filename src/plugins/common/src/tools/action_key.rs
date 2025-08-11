pub mod camera_key;
pub mod movement;
pub mod save_key;
pub mod slot;
pub mod user_input;

use crate::{
	states::menu_state::MenuState,
	tools::{
		action_key::save_key::SaveKey,
		iter_helpers::{first, next},
	},
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
use slot::PlayerSlot;
use std::marker::PhantomData;
use user_input::UserInput;

#[derive(TypePath, Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum ActionKey {
	Movement(MovementKey),
	Slot(PlayerSlot),
	Menu(MenuState),
	Camera(CameraKey),
	Save(SaveKey),
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
			ActionKey::Save(key) => Self::from(key),
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
			ActionKey::Save(key) => Self::from(key),
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
			ActionKey::Camera(key) => next(ActionKey::Camera, key).or(first(ActionKey::Save)),
			ActionKey::Save(key) => next(ActionKey::Save, key),
		}
	}
}

impl InvalidInput for ActionKey {
	type TInput = UserInput;

	fn invalid_input(&self) -> &[UserInput] {
		match self {
			ActionKey::Movement(key) => key.invalid_input(),
			ActionKey::Slot(key) => key.invalid_input(),
			ActionKey::Menu(key) => key.invalid_input(),
			ActionKey::Camera(key) => key.invalid_input(),
			ActionKey::Save(key) => key.invalid_input(),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::IterFinite;
	use std::collections::{HashMap, HashSet};

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(ActionKey::from))
				.chain(PlayerSlot::iterator().map(ActionKey::from))
				.chain(MenuState::iterator().map(ActionKey::from))
				.chain(CameraKey::iterator().map(ActionKey::from))
				.chain(SaveKey::iterator().map(ActionKey::from))
				.collect::<Vec<_>>(),
			ActionKey::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn map_keys_to_user_input() {
		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(UserInput::from))
				.chain(PlayerSlot::iterator().map(UserInput::from))
				.chain(MenuState::iterator().map(UserInput::from))
				.chain(CameraKey::iterator().map(UserInput::from))
				.chain(SaveKey::iterator().map(UserInput::from))
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
			TKey: Into<ActionKey> + InvalidInput<TInput = UserInput> + Copy,
		{
			(key.into(), key.invalid_input().to_vec())
		}

		assert_eq!(
			std::iter::empty()
				.chain(MovementKey::iterator().map(pair_with_invalid_input))
				.chain(PlayerSlot::iterator().map(pair_with_invalid_input))
				.chain(MenuState::iterator().map(pair_with_invalid_input))
				.chain(CameraKey::iterator().map(pair_with_invalid_input))
				.chain(SaveKey::iterator().map(pair_with_invalid_input))
				.collect::<HashMap<_, _>>(),
			ActionKey::iterator()
				.map(pair_with_invalid_input)
				.collect::<HashMap<_, _>>(),
		);
	}
}
