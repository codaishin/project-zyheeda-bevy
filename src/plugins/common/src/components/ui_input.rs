use crate::tools::action_key::user_input::UserInput;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Interaction)]
pub struct UiInput {
	pub(crate) key: UserInput,
	pub(crate) state: UiInputState,
}

impl UiInput {
	pub fn new(key: UserInput) -> Self {
		Self {
			key,
			state: UiInputState::default(),
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum UiInputState {
	#[default]
	None,
	Hovered,
	Primed,
	JustTriggered,
	Triggered,
	JustReleased,
}
