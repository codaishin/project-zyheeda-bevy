use crate::{items::slot_key::SlotKey, resources::SlotMap, traits::InputState};
use bevy::{ecs::schedule::State, input::keyboard::KeyCode};
use common::states::MouseContext;

impl InputState<KeyCode> for State<MouseContext<KeyCode>> {
	fn just_pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
		let MouseContext::JustTriggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}

	fn pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
		let MouseContext::Triggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}

	fn just_released_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
		let MouseContext::JustReleased(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}
}

fn get_slot_key(map: &SlotMap<KeyCode>, key: &KeyCode) -> Vec<SlotKey> {
	let Some(slot_key) = map.slots.get(key) else {
		return vec![];
	};
	vec![*slot_key]
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::components::Side;
	use std::collections::HashSet;

	#[test]
	fn get_just_pressed() {
		let input = State::new(MouseContext::JustTriggered(KeyCode::KeyC));
		let slot_map = SlotMap::new([
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Main)]),
			HashSet::from_iter(input.just_pressed_slots(&slot_map)),
		)
	}

	#[test]
	fn get_pressed() {
		let input = State::new(MouseContext::Triggered(KeyCode::KeyC));
		let slot_map = SlotMap::new([
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Main),]),
			HashSet::from_iter(input.pressed_slots(&slot_map)),
		)
	}

	#[test]
	fn get_just_released() {
		let input = State::new(MouseContext::JustReleased(KeyCode::KeyC));
		let slot_map = SlotMap::new([
			(KeyCode::KeyC, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyD, SlotKey::Hand(Side::Off), ""),
		]);

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Main)]),
			HashSet::from_iter(input.just_released_slots(&slot_map)),
		)
	}
}
