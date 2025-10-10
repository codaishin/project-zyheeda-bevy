use super::GetUserInputState;
use bevy::prelude::*;
use common::{tools::action_key::user_input::UserInput, traits::handles_input::InputState};

impl GetUserInputState for ButtonInput<UserInput> {
	fn get_user_input_state(&self, user_input: UserInput) -> InputState {
		if self.just_pressed(user_input) {
			return InputState::Pressed { just_now: true };
		}

		if self.pressed(user_input) {
			return InputState::Pressed { just_now: false };
		}

		if self.just_released(user_input) {
			return InputState::Released { just_now: true };
		}

		InputState::Released { just_now: false }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::set_input;

	#[test]
	fn just_pressed() {
		let mut input = ButtonInput::default();
		set_input!(input, just_pressed(UserInput::KeyCode(KeyCode::Enter)));

		let state = input.get_user_input_state(UserInput::KeyCode(KeyCode::Enter));

		assert_eq!(InputState::Pressed { just_now: true }, state);
	}

	#[test]
	fn pressed() {
		let mut input = ButtonInput::default();
		set_input!(input, pressed(UserInput::KeyCode(KeyCode::Enter)));

		let state = input.get_user_input_state(UserInput::KeyCode(KeyCode::Enter));

		assert_eq!(InputState::Pressed { just_now: false }, state);
	}

	#[test]
	fn just_released() {
		let mut input = ButtonInput::default();
		set_input!(input, just_released(UserInput::KeyCode(KeyCode::Enter)));

		let state = input.get_user_input_state(UserInput::KeyCode(KeyCode::Enter));

		assert_eq!(InputState::Released { just_now: true }, state);
	}

	#[test]
	fn released() {
		let mut input = ButtonInput::default();
		set_input!(input, released(UserInput::KeyCode(KeyCode::Enter)));

		let state = input.get_user_input_state(UserInput::KeyCode(KeyCode::Enter));

		assert_eq!(InputState::Released { just_now: false }, state);
	}
}
