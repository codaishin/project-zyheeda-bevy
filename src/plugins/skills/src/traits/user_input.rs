use super::{InputState, ShouldEnqueue};
use bevy::input::{ButtonInput, keyboard::KeyCode};
use common::{
	tools::keys::{slot::SlotKey, user_input::UserInput},
	traits::key_mappings::TryGetKey,
};

impl<TMap> InputState<TMap, UserInput> for ButtonInput<UserInput>
where
	TMap: TryGetKey<UserInput, SlotKey>,
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

impl ShouldEnqueue for ButtonInput<UserInput> {
	fn should_enqueue(&self) -> bool {
		self.pressed(UserInput::from(KeyCode::ShiftLeft))
	}
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

	#[test]
	fn get_just_released() {
		let mut input = ButtonInput::<UserInput>::default();
		input.press(UserInput::from(KeyCode::KeyA));
		input.press(UserInput::from(KeyCode::KeyB));
		input.press(UserInput::from(KeyCode::KeyC));
		input.press(UserInput::from(KeyCode::KeyD));
		input.release(UserInput::from(KeyCode::KeyA));
		input.release(UserInput::from(KeyCode::KeyB));
		input.release(UserInput::from(KeyCode::KeyC));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyA));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyB));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyC));
		input.clear_just_pressed(UserInput::from(KeyCode::KeyD));

		assert_eq!(
			HashSet::from([SlotKey::BottomHand(Side::Right),]),
			HashSet::from_iter(input.just_released_slots(&_Map)),
		)
	}

	#[test]
	fn should_enqueue_false() {
		let input = ButtonInput::<UserInput>::default();

		assert!(!input.should_enqueue());
	}

	#[test]
	fn should_enqueue_true() {
		let mut input = ButtonInput::<UserInput>::default();
		input.press(UserInput::from(KeyCode::ShiftLeft));
		input.clear_just_pressed(UserInput::from(KeyCode::ShiftLeft));

		assert!(input.should_enqueue());
	}
}
