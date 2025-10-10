use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::thread_safe::ThreadSafe,
};
use bevy::prelude::*;
use std::hash::Hash;

pub trait GetInput {
	fn get_input<TAction>(&self, value: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait TryGetAction {
	fn try_get_action<TAction>(&self, value: UserInput) -> Option<TAction>
	where
		TAction: Copy + TryFrom<ActionKey> + 'static;
}

pub trait Pressed<TAction> {
	fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> Pressed<TAction> for T
where
	T: TryGetAction,
	TAction: Copy + TryFrom<ActionKey> + 'static,
{
	fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction> {
		input
			.get_pressed()
			.filter_map(|key| self.try_get_action::<TAction>(*key))
	}
}

pub trait JustPressed<TAction> {
	fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> JustPressed<TAction> for T
where
	T: TryGetAction,
	TAction: Copy + TryFrom<ActionKey> + 'static,
{
	fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction> {
		input
			.get_just_pressed()
			.filter_map(|key| self.try_get_action::<TAction>(*key))
	}
}

pub trait JustReleased<TAction> {
	fn just_released(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction>;
}

impl<T, TAction> JustReleased<TAction> for T
where
	T: TryGetAction,
	TAction: Copy + TryFrom<ActionKey> + 'static,
{
	fn just_released(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = TAction> {
		input
			.get_just_released()
			.filter_map(|key| self.try_get_action::<TAction>(*key))
	}
}

pub trait HashCopySafe: Eq + Hash + Copy + ThreadSafe {}

impl<T> HashCopySafe for T where T: Eq + Hash + Copy + ThreadSafe {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::action_key::{movement::MovementKey, user_input::UserInput};
	use std::collections::HashSet;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		Forward,
		Backward,
	}

	impl TryFrom<ActionKey> for _Key {
		type Error = ();

		fn try_from(key: ActionKey) -> Result<Self, ()> {
			match key {
				ActionKey::Movement(MovementKey::Forward) => Ok(_Key::Forward),
				ActionKey::Movement(MovementKey::Backward) => Ok(_Key::Backward),
				_ => Err(()),
			}
		}
	}

	struct _Map;

	impl TryGetAction for _Map {
		fn try_get_action<TAction>(&self, value: UserInput) -> Option<TAction>
		where
			TAction: TryFrom<ActionKey>,
		{
			match value {
				UserInput::KeyCode(KeyCode::KeyW) => {
					TAction::try_from(ActionKey::Movement(MovementKey::Forward)).ok()
				}
				UserInput::KeyCode(KeyCode::KeyS) => {
					TAction::try_from(ActionKey::Movement(MovementKey::Backward)).ok()
				}
				_ => None,
			}
		}
	}

	#[test]
	fn are_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyW));
		input.press(UserInput::KeyCode(KeyCode::KeyS));

		assert_eq!(
			HashSet::from([_Key::Forward, _Key::Backward]),
			map.pressed(&input).collect()
		);
	}

	#[test]
	fn are_just_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyW));
		input.press(UserInput::KeyCode(KeyCode::KeyS));
		input.clear_just_pressed(UserInput::KeyCode(KeyCode::KeyW));

		assert_eq!(
			HashSet::from([_Key::Backward]),
			map.just_pressed(&input).collect()
		);
	}

	#[test]
	fn are_just_released() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(UserInput::KeyCode(KeyCode::KeyW));
		input.press(UserInput::KeyCode(KeyCode::KeyS));
		input.release_all();

		assert_eq!(
			HashSet::from([_Key::Forward, _Key::Backward]),
			map.just_released(&input).collect()
		);
	}
}
