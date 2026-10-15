use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[require(Interaction)]
pub(crate) struct UiButtonState(pub(crate) InputState);

impl Default for UiButtonState {
	fn default() -> Self {
		Self(InputState::Released { just_now: false })
	}
}
