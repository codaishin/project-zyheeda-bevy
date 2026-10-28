use bevy::{
	prelude::*,
	ui_widgets::{ActivateOnPress, Button},
};
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Button, ActivateOnPress)]
pub struct ActionKeyInteraction {
	pub(crate) action_key: ActionKey,
	pub(crate) override_active: bool,
}

impl From<ActionKey> for ActionKeyInteraction {
	fn from(action_key: ActionKey) -> Self {
		Self {
			action_key,
			override_active: false,
		}
	}
}

impl View<MouseOverrideActive> for ActionKeyInteraction {
	fn view(&self) -> bool {
		self.override_active
	}
}
