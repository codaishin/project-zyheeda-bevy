use crate::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{FiniteIter, IterFinite},
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum MenuKey {
	Inventory,
	ComboOverview,
	Settings,
}

impl From<MenuKey> for UserInput {
	fn from(menu_key: MenuKey) -> Self {
		match menu_key {
			MenuKey::Inventory => Self::from(KeyCode::KeyI),
			MenuKey::ComboOverview => Self::from(KeyCode::KeyK),
			MenuKey::Settings => Self::from(KeyCode::Escape),
		}
	}
}

impl From<MenuKey> for Token {
	fn from(menu_key: MenuKey) -> Self {
		match menu_key {
			MenuKey::Inventory => Token::from("menu-inventory"),
			MenuKey::ComboOverview => Token::from("menu-combos"),
			MenuKey::Settings => Token::from("menu-settings"),
		}
	}
}

impl IterFinite for MenuKey {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::Inventory))
	}

	fn next(FiniteIter(current): &FiniteIter<Self>) -> Option<Self> {
		match current.as_ref()? {
			Self::Inventory => Some(Self::ComboOverview),
			Self::ComboOverview => Some(Self::Settings),
			Self::Settings => None,
		}
	}
}

impl InvalidUserInput for MenuKey {
	fn invalid_input(&self) -> &[UserInput] {
		const { &[UserInput::MouseButton(MouseButton::Left)] }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate() {
		assert_eq!(
			vec![
				MenuKey::Inventory,
				MenuKey::ComboOverview,
				MenuKey::Settings
			],
			MenuKey::iterator().take(100).collect::<Vec<_>>(),
		)
	}
}
