use bevy::prelude::*;
use common::{
	tools::action_key::ActionKey,
	traits::{accessors::get::GetProperty, handles_input::MouseOverride},
};

#[derive(Component, Debug, PartialEq)]
#[require(Interaction)]
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

impl GetProperty<MouseOverride> for ActionKeyInteraction {
	fn get_property(&self) -> bool {
		self.override_active
	}
}
