use super::{InputState, ShouldEnqueue};
use crate::{components::SlotKey, resources::SlotMap};
use bevy::input::{keyboard::KeyCode, ButtonInput};
use std::hash::Hash;

impl<TKey: Eq + Hash + Copy + Send + Sync> InputState<TKey> for ButtonInput<TKey> {
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

	#[test]
	fn get_just_pressed() {
		let mut input = ButtonInput::<KeyCode>::default();
		input.press(KeyCode::KeyA);
		input.press(KeyCode::KeyB);
		input.press(KeyCode::KeyC);
		input.press(KeyCode::KeyD);
		input.clear_just_pressed(KeyCode::KeyD);
		let slot_map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::SkillSpawn, ""),
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)]),
			HashSet::from_iter(input.just_pressed_slots(&slot_map)),
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
		let slot_map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::SkillSpawn, ""),
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([
				SlotKey::SkillSpawn,
				SlotKey::Hand(Side::Main),
				SlotKey::Hand(Side::Off),
			]),
			HashSet::from_iter(input.pressed_slots(&slot_map)),
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
		let slot_map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::SkillSpawn, ""),
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::SkillSpawn, SlotKey::Hand(Side::Main),]),
			HashSet::from_iter(input.just_released_slots(&slot_map)),
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
