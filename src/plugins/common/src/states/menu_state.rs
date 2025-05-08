use super::game_state::GameState;
use crate::{
	tools::action_key::{ActionKey, IsNot, user_input::UserInput},
	traits::{
		handles_localization::Token,
		iteration::{Iter, IterFinite},
	},
};
use bevy::input::keyboard::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum MenuState {
	#[default]
	Inventory,
	ComboOverview,
	Settings,
}

impl From<MenuState> for GameState {
	fn from(menu_state: MenuState) -> Self {
		GameState::IngameMenu(menu_state)
	}
}

impl From<MenuState> for ActionKey {
	fn from(menu_state: MenuState) -> Self {
		Self::Menu(menu_state)
	}
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
			MenuState::Settings => Token::from("menu-settings"),
		}
	}
}

impl TryFrom<GameState> for MenuState {
	type Error = IsNot<MenuState>;

	fn try_from(game_state: GameState) -> Result<Self, Self::Error> {
		match game_state {
			GameState::IngameMenu(menu_state) => Ok(menu_state),
			_ => Err(IsNot::key()),
		}
	}
}

impl TryFrom<ActionKey> for MenuState {
	type Error = IsNot<MenuState>;

	fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
		match key {
			ActionKey::Menu(menu_state) => Ok(menu_state),
			_ => Err(IsNot::key()),
		}
	}
}

impl IterFinite for MenuState {
	fn iterator() -> Iter<Self> {
		Iter(Some(MenuState::Inventory))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match &current.0? {
			MenuState::Inventory => Some(MenuState::ComboOverview),
			MenuState::ComboOverview => Some(MenuState::Settings),
			MenuState::Settings => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn menu_keys() {
		assert_eq!(
			vec![
				MenuState::Inventory,
				MenuState::ComboOverview,
				MenuState::Settings
			],
			MenuState::iterator().take(100).collect::<Vec<_>>(),
		);
	}
}
