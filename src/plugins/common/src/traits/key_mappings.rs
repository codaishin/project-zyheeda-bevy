use crate::tools::keys::user_input::UserInput;
use bevy::prelude::*;
use std::hash::Hash;

pub trait GetUserInput<TKey, TUserInput> {
	fn get_key_code(&self, value: TKey) -> TUserInput;
}

pub trait TryGetKey<TUserInput, TKey> {
	fn try_get_key(&self, value: TUserInput) -> Option<TKey>;
}

pub trait Pressed<TKey> {
	fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey> Pressed<TKey> for T
where
	T: TryGetKey<UserInput, TKey>,
	TKey: Eq + Hash,
{
	fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey> {
		input.get_pressed().filter_map(|key| self.try_get_key(*key))
	}
}

pub trait JustPressed<TKey> {
	fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey> JustPressed<TKey> for T
where
	T: TryGetKey<UserInput, TKey>,
	TKey: Eq + Hash,
{
	fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey> {
		input
			.get_just_pressed()
			.filter_map(|key| self.try_get_key(*key))
	}
}

pub trait JustReleased<TKey> {
	fn just_released(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey> JustReleased<TKey> for T
where
	T: TryGetKey<UserInput, TKey>,
	TKey: Eq + Hash,
{
	fn just_released(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TKey> {
		input
			.get_just_released()
			.filter_map(|key| self.try_get_key(*key))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		A,
		B,
	}

	struct _Map;

	impl TryGetKey<UserInput, _Key> for _Map {
		fn try_get_key(&self, value: UserInput) -> Option<_Key> {
			match value {
				UserInput::KeyCode(KeyCode::KeyA) => Some(_Key::A),
				UserInput::KeyCode(KeyCode::KeyB) => Some(_Key::B),
				_ => None,
			}
		}
	}

	#[test]
	fn are_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyA));
		input.press(UserInput::KeyCode(KeyCode::KeyB));

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.pressed(&input).collect()
		);
	}

	#[test]
	fn are_just_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyA));
		input.press(UserInput::KeyCode(KeyCode::KeyB));
		input.clear_just_pressed(UserInput::KeyCode(KeyCode::KeyA));

		assert_eq!(HashSet::from([_Key::B]), map.just_pressed(&input).collect());
	}

	#[test]
	fn are_just_released() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyA));
		input.press(UserInput::KeyCode(KeyCode::KeyB));
		input.release_all();

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.just_released(&input).collect()
		);
	}
}
