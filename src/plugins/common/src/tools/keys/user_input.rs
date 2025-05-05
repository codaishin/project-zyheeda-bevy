use super::IsNot;
use crate::traits::handles_localization::Token;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum UserInput {
	KeyCode(KeyCode),
	MouseButton(MouseButton),
}

impl From<UserInput> for Token {
	fn from(value: UserInput) -> Self {
		match value {
			UserInput::KeyCode(key_code) => Self::from(key_code),
			UserInput::MouseButton(mouse_button) => Self::from(mouse_button),
		}
	}
}

impl From<KeyCode> for UserInput {
	fn from(key_code: KeyCode) -> Self {
		Self::KeyCode(key_code)
	}
}

impl TryFrom<UserInput> for KeyCode {
	type Error = IsNot<KeyCode>;

	fn try_from(user_input: UserInput) -> Result<Self, Self::Error> {
		let UserInput::KeyCode(key_code) = user_input else {
			return Err(IsNot::key());
		};

		Ok(key_code)
	}
}

impl From<MouseButton> for UserInput {
	fn from(mouse_button: MouseButton) -> Self {
		Self::MouseButton(mouse_button)
	}
}

impl TryFrom<UserInput> for MouseButton {
	type Error = IsNot<MouseButton>;

	fn try_from(user_input: UserInput) -> Result<Self, Self::Error> {
		let UserInput::MouseButton(mouse_button) = user_input else {
			return Err(IsNot::key());
		};

		Ok(mouse_button)
	}
}
