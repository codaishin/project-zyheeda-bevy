use super::{InputState, ShouldEnqueue};
use crate::slot_key::SlotKey;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use common::traits::map_value::TryMapBackwards;
use std::hash::Hash;

impl<TMap: TryMapBackwards<TKey, SlotKey>, TKey: Eq + Hash + Copy + Send + Sync>
	InputState<TMap, TKey> for ButtonInput<TKey>
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_just_pressed()
			.filter_map(|k| map.try_map_backwards(*k))
			.collect()
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_pressed()
			.filter_map(|k| map.try_map_backwards(*k))
			.collect()
	}

	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_just_released()
			.filter_map(|k| map.try_map_backwards(*k))
			.collect()
	}
}

impl ShouldEnqueue for ButtonInput<KeyCode> {
	fn should_enqueue(&self) -> bool {
		self.pressed(KeyCode::ShiftLeft)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::components::Side;
	use std::collections::HashSet;

	struct _Map;

	impl TryMapBackwards<KeyCode, SlotKey> for _Map {
		fn try_map_backwards(&self, value: KeyCode) -> Option<SlotKey> {
			match value {
				KeyCode::KeyC => Some(SlotKey::BottomHand(Side::Right)),
				KeyCode::KeyD => Some(SlotKey::BottomHand(Side::Left)),
				_ => None,
			}
		}
	}

	#[test]
	fn get_just_pressed() {
		let mut input = ButtonInput::<KeyCode>::default();
		input.press(KeyCode::KeyA);
		input.press(KeyCode::KeyB);
		input.press(KeyCode::KeyC);
		input.press(KeyCode::KeyD);
		input.clear_just_pressed(KeyCode::KeyD);

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right)]),
			HashSet::from_iter(input.just_pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_pressed() {
		let mut input = ButtonInput::<KeyCode>::default();
		input.press(KeyCode::KeyA);
		input.press(KeyCode::KeyB);
		input.press(KeyCode::KeyC);
		input.press(KeyCode::KeyD);
		input.clear_just_pressed(KeyCode::KeyA);
		input.clear_just_pressed(KeyCode::KeyB);
		input.clear_just_pressed(KeyCode::KeyC);
		input.clear_just_pressed(KeyCode::KeyD);

		assert_eq!(
			HashSet::from([
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			]),
			HashSet::from_iter(input.pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_just_released() {
		let mut input = ButtonInput::<KeyCode>::default();
		input.press(KeyCode::KeyA);
		input.press(KeyCode::KeyB);
		input.press(KeyCode::KeyC);
		input.press(KeyCode::KeyD);
		input.release(KeyCode::KeyA);
		input.release(KeyCode::KeyB);
		input.release(KeyCode::KeyC);
		input.clear_just_pressed(KeyCode::KeyA);
		input.clear_just_pressed(KeyCode::KeyB);
		input.clear_just_pressed(KeyCode::KeyC);
		input.clear_just_pressed(KeyCode::KeyD);

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right),]),
			HashSet::from_iter(input.just_released_slots(&_Map)),
		)
	}

	#[test]
	fn should_enqueue_false() {
		let input = ButtonInput::<KeyCode>::default();

		assert!(!input.should_enqueue());
	}

	#[test]
	fn should_enqueue_true() {
		let mut input = ButtonInput::<KeyCode>::default();
		input.press(KeyCode::ShiftLeft);
		input.clear_just_pressed(KeyCode::ShiftLeft);

		assert!(input.should_enqueue());
	}
}
