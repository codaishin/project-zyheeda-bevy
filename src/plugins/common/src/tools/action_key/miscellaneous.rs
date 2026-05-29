use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{Iter, IterFinite},
	},
};
use bevy::input::keyboard::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Miscellaneous {
	Interact,
}

impl InvalidUserInput for Miscellaneous {
	fn invalid_input(&self) -> &[UserInput] {
		&[]
	}
}

impl From<Miscellaneous> for ActionKey {
	fn from(target: Miscellaneous) -> Self {
		Self::Miscellaneous(target)
	}
}

impl From<Miscellaneous> for UserInput {
	fn from(_: Miscellaneous) -> Self {
		Self::KeyCode(KeyCode::KeyF)
	}
}

impl From<Miscellaneous> for Token {
	fn from(_: Miscellaneous) -> Self {
		Self::from("interact")
	}
}

impl IterFinite for Miscellaneous {
	fn iterator() -> Iter<Self> {
		Iter(Some(Miscellaneous::Interact))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Miscellaneous::Interact => None,
		}
	}
}
