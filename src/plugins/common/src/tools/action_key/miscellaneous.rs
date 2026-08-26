use crate::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{FiniteIter, IterFinite},
	},
};
use bevy::input::keyboard::KeyCode;
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Miscellaneous {
	Interact,
	Paused,
}

impl InvalidUserInput for Miscellaneous {
	fn invalid_input(&self) -> &[UserInput] {
		&[]
	}
}

impl From<Miscellaneous> for UserInput {
	fn from(m: Miscellaneous) -> Self {
		match m {
			Miscellaneous::Interact => Self::KeyCode(KeyCode::KeyF),
			Miscellaneous::Paused => Self::KeyCode(KeyCode::KeyP),
		}
	}
}

impl From<Miscellaneous> for Token {
	fn from(m: Miscellaneous) -> Self {
		match m {
			Miscellaneous::Interact => Self::from("interact"),
			Miscellaneous::Paused => Self::from("paused"),
		}
	}
}

impl IterFinite for Miscellaneous {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Miscellaneous::Interact))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			Miscellaneous::Interact => Some(Miscellaneous::Paused),
			Miscellaneous::Paused => None,
		}
	}
}
