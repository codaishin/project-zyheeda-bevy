use crate::traits::InputState;
use bevy::{input::keyboard::KeyCode, state::state::State};
use common::{
	states::mouse_context::MouseContext,
	tools::keys::slot::SlotKey,
	traits::key_mappings::TryGetKey,
};

impl<TMap: TryGetKey<KeyCode, SlotKey>> InputState<TMap, KeyCode> for State<MouseContext<KeyCode>> {
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::JustTriggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::Triggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}

	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::JustReleased(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, key)
	}
}

fn get_slot_key<TMap: TryGetKey<KeyCode, SlotKey>>(map: &TMap, key: &KeyCode) -> Vec<SlotKey> {
	let Some(slot_key) = map.try_get_key(*key) else {
		return vec![];
	};
	vec![slot_key]
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::tools::keys::slot::Side;
	use std::collections::HashSet;

	struct _Map;

	impl TryGetKey<KeyCode, SlotKey> for _Map {
		fn try_get_key(&self, value: KeyCode) -> Option<SlotKey> {
			match value {
				KeyCode::KeyC => Some(SlotKey::BottomHand(Side::Right)),
				KeyCode::KeyD => Some(SlotKey::BottomHand(Side::Right)),
				_ => None,
			}
		}
	}

	#[test]
	fn get_just_pressed() {
		let input = State::new(MouseContext::JustTriggered(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right)]),
			HashSet::from_iter(input.just_pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_pressed() {
		let input = State::new(MouseContext::Triggered(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right),]),
			HashSet::from_iter(input.pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_just_released() {
		let input = State::new(MouseContext::JustReleased(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right)]),
			HashSet::from_iter(input.just_released_slots(&_Map)),
		)
	}
}
