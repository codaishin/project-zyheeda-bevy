mod button_input;

use common::{tools::action_key::user_input::UserInput, traits::handles_input::InputState};

pub(crate) trait GetUserInputState {
	fn get_user_input_state(&self, user_input: UserInput) -> InputState;
}
