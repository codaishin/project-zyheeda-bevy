use crate::traits::InputState;
use bevy::{input::keyboard::KeyCode, state::state::State};
use common::{
	states::mouse_context::MouseContext,
	tools::keys::{slot::SlotKey, user_input::UserInput},
	traits::key_mappings::TryGetKey,
};

impl<TMap> InputState<TMap, UserInput> for State<MouseContext<KeyCode>>
where
	TMap: TryGetKey<UserInput, SlotKey>,
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::JustTriggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, UserInput::from(*key))
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::Triggered(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, UserInput::from(*key))
	}

	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey> {
		let MouseContext::JustReleased(key) = self.get() else {
			return vec![];
		};
		get_slot_key(map, UserInput::from(*key))
	}
}

fn get_slot_key<TMap>(map: &TMap, user_input: UserInput) -> Vec<SlotKey>
where
	TMap: TryGetKey<UserInput, SlotKey>,
{
	let Some(slot_key) = map.try_get_key(user_input) else {
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

	impl TryGetKey<UserInput, SlotKey> for _Map {
		fn try_get_key(&self, value: UserInput) -> Option<SlotKey> {
			match value {
				UserInput::KeyCode(KeyCode::KeyC) => Some(SlotKey::BottomHand(Side::Right)),
				UserInput::KeyCode(KeyCode::KeyD) => Some(SlotKey::BottomHand(Side::Right)),
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
