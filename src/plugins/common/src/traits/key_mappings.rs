use super::thread_safe::ThreadSafe;
use bevy::input::ButtonInput;
use std::hash::Hash;

pub trait GetUserInput<TKey, TUserInput> {
	fn get_key_code(&self, value: TKey) -> TUserInput;
}

pub trait TryGetKey<TUserInput, TKey> {
	fn try_get_key(&self, value: TUserInput) -> Option<TKey>;
}

pub trait MapKey: Into<Self::TMapKey> {
	type TMapKey;
}

pub trait Pressed<TKey, TBevyInput>
where
	TBevyInput: Eq + Hash + Copy + ThreadSafe,
{
	fn pressed(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TBevyInput> Pressed<TKey, TBevyInput> for T
where
	T: TryGetKey<TBevyInput::TMapKey, TKey>,
	TKey: Eq + Hash,
	TBevyInput: MapKey + Eq + Hash + Copy + ThreadSafe,
{
	fn pressed(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey> {
		input
			.get_pressed()
			.filter_map(|key| self.try_get_key((*key).into()))
	}
}

pub trait JustPressed<TKey, TBevyInput>
where
	TBevyInput: Eq + Hash + Copy + ThreadSafe,
{
	fn just_pressed(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TBevyInput> JustPressed<TKey, TBevyInput> for T
where
	T: TryGetKey<TBevyInput::TMapKey, TKey>,
	TKey: Eq + Hash,
	TBevyInput: MapKey + Eq + Hash + Copy + ThreadSafe,
{
	fn just_pressed(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey> {
		input
			.get_just_pressed()
			.filter_map(|key| self.try_get_key((*key).into()))
	}
}

pub trait JustReleased<TKey, TBevyInput>
where
	TBevyInput: MapKey + Eq + Hash + Copy + ThreadSafe,
{
	fn just_released(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TBevyInput> JustReleased<TKey, TBevyInput> for T
where
	T: TryGetKey<TBevyInput::TMapKey, TKey>,
	TKey: Eq + Hash,
	TBevyInput: MapKey + Eq + Hash + Copy + ThreadSafe,
{
	fn just_released(&self, input: &ButtonInput<TBevyInput>) -> impl Iterator<Item = TKey> {
		input
			.get_just_released()
			.filter_map(|key| self.try_get_key((*key).into()))
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

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _MapKey {
		A,
		B,
	}

	impl From<_UserInput> for _MapKey {
		fn from(input: _UserInput) -> Self {
			match input {
				_UserInput::A => _MapKey::A,
				_UserInput::B => _MapKey::B,
			}
		}
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _UserInput {
		A,
		B,
	}

	impl MapKey for _UserInput {
		type TMapKey = _MapKey;
	}

	struct _Map;

	impl TryGetKey<_MapKey, _Key> for _Map {
		fn try_get_key(&self, value: _MapKey) -> Option<_Key> {
			match value {
				_MapKey::A => Some(_Key::A),
				_MapKey::B => Some(_Key::B),
			}
		}
	}

	#[test]
	fn are_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_UserInput::A);
		input.press(_UserInput::B);

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.pressed(&input).collect()
		);
	}

	#[test]
	fn are_just_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_UserInput::A);
		input.press(_UserInput::B);
		input.clear_just_pressed(_UserInput::A);

		assert_eq!(HashSet::from([_Key::B]), map.just_pressed(&input).collect());
	}

	#[test]
	fn are_just_released() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_UserInput::A);
		input.press(_UserInput::B);
		input.release_all();

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.just_released(&input).collect()
		);
	}
}
