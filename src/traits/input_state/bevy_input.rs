use crate::{components::SlotKey, resources::SlotMap};

use super::{InputState, ShouldEnqueue};
use bevy::input::{keyboard::KeyCode, Input};
use std::hash::Hash;

impl<TKey: Eq + Hash + Copy + Send + Sync> InputState<TKey> for Input<TKey> {
	fn just_pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey> {
		self.get_just_pressed()
			.filter_map(|k| map.slots.get(k))
			.cloned()
			.collect()
	}

	fn pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey> {
		self.get_pressed()
			.filter_map(|k| map.slots.get(k))
			.cloned()
			.collect()
	}

	fn just_released_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey> {
		self.get_just_released()
			.filter_map(|k| map.slots.get(k))
			.cloned()
			.collect()
	}
}

impl ShouldEnqueue for Input<KeyCode> {
	fn should_enqueue(&self) -> bool {
		self.pressed(KeyCode::ShiftLeft)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Side;
	use bevy::input::keyboard::KeyCode;
	use std::collections::HashSet;

	#[test]
	fn get_just_pressed() {
		let mut input = Input::<KeyCode>::default();
		input.press(KeyCode::A);
		input.press(KeyCode::B);
		input.press(KeyCode::C);
		input.press(KeyCode::D);
		input.clear_just_pressed(KeyCode::D);
		let slot_map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, ""),
			(KeyCode::C, SlotKey::Hand(Side::Main), ""),
			(KeyCode::D, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::Legs, SlotKey::Hand(Side::Main)]),
			HashSet::from_iter(input.just_pressed_slots(&slot_map)),
		)
	}

	#[test]
	fn get_pressed() {
		let mut input = Input::<KeyCode>::default();
		input.press(KeyCode::A);
		input.press(KeyCode::B);
		input.press(KeyCode::C);
		input.press(KeyCode::D);
		input.clear_just_pressed(KeyCode::A);
		input.clear_just_pressed(KeyCode::B);
		input.clear_just_pressed(KeyCode::C);
		input.clear_just_pressed(KeyCode::D);
		let slot_map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, ""),
			(KeyCode::C, SlotKey::Hand(Side::Main), ""),
			(KeyCode::D, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([
				SlotKey::Legs,
				SlotKey::Hand(Side::Main),
				SlotKey::Hand(Side::Off),
			]),
			HashSet::from_iter(input.pressed_slots(&slot_map)),
		)
	}
	#[test]
	fn get_just_released() {
		let mut input = Input::<KeyCode>::default();
		input.press(KeyCode::A);
		input.press(KeyCode::B);
		input.press(KeyCode::C);
		input.press(KeyCode::D);
		input.release(KeyCode::A);
		input.release(KeyCode::B);
		input.release(KeyCode::C);
		input.clear_just_pressed(KeyCode::A);
		input.clear_just_pressed(KeyCode::B);
		input.clear_just_pressed(KeyCode::C);
		input.clear_just_pressed(KeyCode::D);
		let slot_map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, ""),
			(KeyCode::C, SlotKey::Hand(Side::Main), ""),
			(KeyCode::D, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::Legs, SlotKey::Hand(Side::Main),]),
			HashSet::from_iter(input.just_released_slots(&slot_map)),
		)
	}

	#[test]
	fn should_enqueue_false() {
		let input = Input::<KeyCode>::default();

		assert!(!input.should_enqueue());
	}

	#[test]
	fn should_enqueue_true() {
		let mut input = Input::<KeyCode>::default();
		input.press(KeyCode::ShiftLeft);
		input.clear_just_pressed(KeyCode::ShiftLeft);

		assert!(input.should_enqueue());
	}
}
