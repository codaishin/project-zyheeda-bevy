use crate::{tools::keys::user_input::UserInput, traits::handles_localization::Token};
use bevy::input::keyboard::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum MenuState {
	#[default]
	Inventory,
	ComboOverview,
	Settings,
}

impl From<MenuState> for UserInput {
	fn from(menu_state: MenuState) -> Self {
		match menu_state {
			MenuState::Inventory => Self::from(KeyCode::KeyI),
			MenuState::ComboOverview => Self::from(KeyCode::KeyK),
			MenuState::Settings => Self::from(KeyCode::Escape),
		}
	}
}

impl From<MenuState> for Token {
	fn from(menu_state: MenuState) -> Self {
		match menu_state {
			MenuState::Inventory => Token::from("menu-inventory"),
			MenuState::ComboOverview => Token::from("menu-combos"),
			MenuState::Settings => Token::from("menu=settings"),
		}
	}
}
