use super::InputState;
use bevy::input::ButtonInput;
use common::{
	tools::action_key::{slot::SlotKey, user_input::UserInput},
	traits::key_mappings::TryGetAction,
};

impl<TMap> InputState<TMap, UserInput> for ButtonInput<UserInput>
where
	TMap: TryGetAction<UserInput, SlotKey>,
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_just_pressed()
			.filter_map(|k| map.try_get_action(*k))
			.collect()
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey> {
		self.get_pressed()
			.filter_map(|k| map.try_get_action(*k))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::tools::action_key::slot::Side;
	use std::collections::HashSet;

	struct _Map;

	impl TryGetAction<UserInput, SlotKey> for _Map {
		fn try_get_action(&self, value: UserInput) -> Option<SlotKey> {
			match value {
				UserInput::KeyCode(KeyCode::KeyC) => Some(SlotKey::BottomHand(Side::Right)),
				UserInput::KeyCode(KeyCode::KeyD) => Some(SlotKey::BottomHand(Side::Left)),
				_ => None,
			}
		}
	}

	#[test]
	fn get_just_pressed() {
		let mut input = ButtonInput::<UserInput>::default();
		input.press(UserInput::from(KeyCode::KeyA));
		input.press(UserInput::from(KeyCode::KeyB));
		input.press(UserInput::from(KeyCode::KeyC));
		input.press(UserInput::from(KeyCode::KeyD));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyD));

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right)]),
			HashSet::from_iter(input.just_pressed_slots(&_Map)),
		)
	}

	#[test]
	fn get_pressed() {
		let mut input = ButtonInput::<UserInput>::default();
		input.press(UserInput::from(KeyCode::KeyA));
		input.press(UserInput::from(KeyCode::KeyB));
		input.press(UserInput::from(KeyCode::KeyC));
		input.press(UserInput::from(KeyCode::KeyD));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyA));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyB));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyC));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyD));

		assert_eq!(
			HashSet::from([
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			]),
			HashSet::from_iter(input.pressed_slots(&_Map)),
		)
	}
}
