use super::thread_safe::ThreadSafe;
use bevy::input::ButtonInput;
use std::hash::Hash;

pub trait GetKeyCode<TKey, TKeyCode> {
	fn get_key_code(&self, value: TKey) -> TKeyCode;
}

pub trait TryGetKey<TKeyCode, TKey> {
	fn try_get_key(&self, value: TKeyCode) -> Option<TKey>;
}

pub trait Pressed<TKey, TKeyCode>
where
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn pressed(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TKeyCode> Pressed<TKey, TKeyCode> for T
where
	T: TryGetKey<TKeyCode, TKey>,
	TKey: Eq + Hash,
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn pressed(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey> {
		input.get_pressed().filter_map(|key| self.try_get_key(*key))
	}
}

pub trait JustPressed<TKey, TKeyCode>
where
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn just_pressed(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TKeyCode> JustPressed<TKey, TKeyCode> for T
where
	T: TryGetKey<TKeyCode, TKey>,
	TKey: Eq + Hash,
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn just_pressed(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey> {
		input
			.get_just_pressed()
			.filter_map(|key| self.try_get_key(*key))
	}
}

pub trait JustReleased<TKey, TKeyCode>
where
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn just_released(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey>;
}

impl<T, TKey, TKeyCode> JustReleased<TKey, TKeyCode> for T
where
	T: TryGetKey<TKeyCode, TKey>,
	TKey: Eq + Hash,
	TKeyCode: Eq + Hash + Copy + ThreadSafe,
{
	fn just_released(&self, input: &ButtonInput<TKeyCode>) -> impl Iterator<Item = TKey> {
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

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _KeyCode {
		A,
		B,
	}

	struct _Map;

	impl TryGetKey<_KeyCode, _Key> for _Map {
		fn try_get_key(&self, value: _KeyCode) -> Option<_Key> {
			match value {
				_KeyCode::A => Some(_Key::A),
				_KeyCode::B => Some(_Key::B),
			}
		}
	}

	#[test]
	fn are_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_KeyCode::A);
		input.press(_KeyCode::B);

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.pressed(&input).collect()
		);
	}

	#[test]
	fn are_just_pressed() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_KeyCode::A);
		input.press(_KeyCode::B);
		input.clear_just_pressed(_KeyCode::A);

		assert_eq!(HashSet::from([_Key::B]), map.just_pressed(&input).collect());
	}

	#[test]
	fn are_just_released() {
		let map = _Map;
		let mut input = ButtonInput::default();

		input.press(_KeyCode::A);
		input.press(_KeyCode::B);
		input.release_all();

		assert_eq!(
			HashSet::from([_Key::A, _Key::B]),
			map.just_released(&input).collect()
		);
	}
}
