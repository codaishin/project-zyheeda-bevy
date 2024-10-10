use crate::{items::slot_key::SlotKey, traits::InputState};
use bevy::{input::keyboard::KeyCode, state::state::State};
use common::{states::MouseContext, traits::map_value::TryMapBackwards};

impl<TMap: TryMapBackwards<KeyCode, SlotKey>> InputState<TMap, KeyCode>
	for State<MouseContext<KeyCode>>
{
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

fn get_slot_key<TMap: TryMapBackwards<KeyCode, SlotKey>>(
	map: &TMap,
	key: &KeyCode,
) -> Vec<SlotKey> {
	let Some(slot_key) = map.try_map_backwards(*key) else {
		return vec![];
	};
	vec![slot_key]
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
				KeyCode::KeyC => Some(SlotKey::Hand(Side::Right)),
				KeyCode::KeyD => Some(SlotKey::Hand(Side::Right)),
				_ => None,
			}
		}
	}

	#[test]
	fn get_just_pressed() {
		let input = State::new(MouseContext::JustTriggered(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Right)]),
			HashSet::from_iter(input.just_pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_pressed() {
		let input = State::new(MouseContext::Triggered(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Right),]),
			HashSet::from_iter(input.pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_just_released() {
		let input = State::new(MouseContext::JustReleased(KeyCode::KeyC));

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Right)]),
			HashSet::from_iter(input.just_released_slots(&_Map)),
		)
	}
}
