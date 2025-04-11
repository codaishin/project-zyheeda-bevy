use super::{InputState, ShouldEnqueue};
use bevy::input::{ButtonInput, keyboard::KeyCode};
use common::{tools::slot_key::SlotKey, traits::key_mappings::TryGetKey};
use std::hash::Hash;

impl<TMap: TryGetKey<TKey, SlotKey>, TKey: Eq + Hash + Copy + Send + Sync> InputState<TMap, TKey>
	for ButtonInput<TKey>
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_just_pressed()
			.filter_map(|k| map.try_get_key(*k))
			.collect()
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_pressed()
			.filter_map(|k| map.try_get_key(*k))
			.collect()
	}

	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_just_released()
			.filter_map(|k| map.try_get_key(*k))
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
	use common::tools::slot_key::Side;
	use std::collections::HashSet;

	struct _Map;

	impl TryGetKey<KeyCode, SlotKey> for _Map {
		fn try_get_key(&self, value: KeyCode) -> Option<SlotKey> {
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
