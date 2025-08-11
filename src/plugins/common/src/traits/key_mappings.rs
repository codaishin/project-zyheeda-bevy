use crate::traits::thread_safe::ThreadSafe;
use bevy::prelude::*;
use std::hash::Hash;

pub trait GetInput<TAction> {
	type TInput;

	fn get_input(&self, value: TAction) -> Self::TInput;
}

pub trait TryGetAction<TAction> {
	type TInput;

	fn try_get_action(&self, value: Self::TInput) -> Option<TAction>;
}

pub trait Pressed<TAction> {
	type TInput: HashCopySafe;

	fn pressed(&self, input: &ButtonInput<Self::TInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> Pressed<TAction> for T
where
	T: TryGetAction<TAction, TInput: HashCopySafe>,
{
	type TInput = <T as TryGetAction<TAction>>::TInput;

	fn pressed(&self, input: &ButtonInput<Self::TInput>) -> impl Iterator<Item = TAction> {
		input
			.get_pressed()
			.filter_map(|key| self.try_get_action(*key))
	}
}

pub trait JustPressed<TAction> {
	type TInput: HashCopySafe;

	fn just_pressed(&self, input: &ButtonInput<Self::TInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> JustPressed<TAction> for T
where
	T: TryGetAction<TAction, TInput: HashCopySafe>,
{
	type TInput = <T as TryGetAction<TAction>>::TInput;

	fn just_pressed(&self, input: &ButtonInput<T::TInput>) -> impl Iterator<Item = TAction> {
		input
			.get_just_pressed()
			.filter_map(|key| self.try_get_action(*key))
	}
}

pub trait JustReleased<TAction> {
	type TInput: HashCopySafe;

	fn just_released(&self, input: &ButtonInput<Self::TInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> JustReleased<TAction> for T
where
	T: TryGetAction<TAction, TInput: HashCopySafe>,
{
	type TInput = <T as TryGetAction<TAction>>::TInput;

	fn just_released(&self, input: &ButtonInput<Self::TInput>) -> impl Iterator<Item = TAction> {
		input
			.get_just_released()
			.filter_map(|key| self.try_get_action(*key))
	}
}

pub trait HashCopySafe: Eq + Hash + Copy + ThreadSafe {}

impl<T> HashCopySafe for T where T: Eq + Hash + Copy + ThreadSafe {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::action_key::user_input::UserInput;
	use std::collections::HashSet;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		A,
		B,
	}

	struct _Map;

	impl TryGetAction<_Key> for _Map {
		type TInput = UserInput;

		fn try_get_action(&self, value: UserInput) -> Option<_Key> {
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
