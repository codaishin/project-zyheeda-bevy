pub mod combination;

use crate::{
	tools::action_key::user_input::combination::UserInputCombination,
	traits::{
		accessors::get::ViewField,
		handles_localization::{Token, TokenItem},
	},
};
use bevy::prelude::*;
use macros::serde_model;
use zyheeda_core::prelude::*;

#[serde_model]
#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UserInput {
	KeyCode(KeyCode),
	MouseButton(MouseButton),
	Combined(UserInputCombination<2>),
}

impl From<UserInput> for Token {
	fn from(value: UserInput) -> Self {
		match value {
			UserInput::KeyCode(key) => Token::from_iter([TokenItem::from(key)]),
			UserInput::MouseButton(button) => Token::from_iter([TokenItem::from(button)]),
			UserInput::Combined(combination) => Token::from(combination),
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
			return Err(IsNot::target_type());
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
			return Err(IsNot::target_type());
		};

		Ok(mouse_button)
	}
}

impl ViewField for UserInput {
	type TValue<'a> = Self;
}
