use super::InputState;
use bevy::input::ButtonInput;
use common::traits::key_mappings::{HashCopySafe, TryGetAction};

impl<TMap, TOutput> InputState<TMap, TOutput> for ButtonInput<TMap::TInput>
where
	TMap: TryGetAction<TOutput, TInput: HashCopySafe>,
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<TOutput> {
		self.get_just_pressed()
			.filter_map(|k| map.try_get_action(*k))
			.collect()
	}

	fn pressed_slots(&self, map: &TMap) -> Vec<TOutput> {
		self.get_pressed()
			.filter_map(|k| map.try_get_action(*k))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::tools::action_key::{
		slot::{PlayerSlot, Side},
		user_input::UserInput,
	};
	use std::collections::HashSet;

	struct _Map;

	impl TryGetAction<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn try_get_action(&self, value: UserInput) -> Option<PlayerSlot> {
			match value {
				UserInput::KeyCode(KeyCode::KeyC) => Some(PlayerSlot::Lower(Side::Right)),
				UserInput::KeyCode(KeyCode::KeyD) => Some(PlayerSlot::Lower(Side::Left)),
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
			HashSet::from([PlayerSlot::Lower(Side::Right)]),
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
				PlayerSlot::Lower(Side::Right),
				PlayerSlot::Lower(Side::Left),
			]),
			HashSet::from_iter(input.pressed_slots(&_Map)),
		)
	}
}
