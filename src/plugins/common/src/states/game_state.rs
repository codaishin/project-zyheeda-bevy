use super::menu_state::MenuState;
use crate::traits::{
	iteration::{Iter, IterFinite},
	states::PlayState,
};
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
	#[default]
	None,
	StartMenu,
	Loading,
	NewGame,
	Play,
	Saving,
	IngameMenu(MenuState),
}

#[derive(Debug, PartialEq)]
pub struct NoKeySet;

impl TryFrom<GameState> for KeyCode {
	type Error = NoKeySet;

	fn try_from(value: GameState) -> Result<Self, Self::Error> {
		match value {
			GameState::Saving => Ok(KeyCode::F5),
			GameState::IngameMenu(MenuState::Inventory) => Ok(KeyCode::KeyI),
			GameState::IngameMenu(MenuState::ComboOverview) => Ok(KeyCode::KeyK),
			_ => Err(NoKeySet),
		}
	}
}

impl IterFinite for GameState {
	fn iterator() -> Iter<Self> {
		Iter(Some(GameState::None))
	}

	fn next(Iter(current): &Iter<Self>) -> Option<Self> {
		match current.as_ref()? {
			GameState::None => Some(GameState::StartMenu),
			GameState::StartMenu => Some(GameState::NewGame),
			GameState::NewGame => Some(GameState::Loading),
			GameState::Loading => Some(GameState::Play),
			GameState::Play => Some(GameState::Saving),
			GameState::Saving => Some(GameState::IngameMenu(MenuState::Inventory)),
			GameState::IngameMenu(MenuState::Inventory) => {
				Some(GameState::IngameMenu(MenuState::ComboOverview))
			}
			GameState::IngameMenu(MenuState::ComboOverview) => None,
		}
	}
}

impl PlayState for GameState {
	fn play_state() -> Self {
		Self::Play
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_all_states() {
		assert_eq!(
			vec![
				GameState::None,
				GameState::StartMenu,
				GameState::NewGame,
				GameState::Loading,
				GameState::Play,
				GameState::Saving,
				GameState::IngameMenu(MenuState::Inventory),
				GameState::IngameMenu(MenuState::ComboOverview)
			],
			GameState::iterator().collect::<Vec<_>>(),
		)
	}

	#[test]
	fn get_key_codes() {
		assert_eq!(
			vec![
				Err(NoKeySet),
				Err(NoKeySet),
				Err(NoKeySet),
				Err(NoKeySet),
				Err(NoKeySet),
				Ok(KeyCode::F5),
				Ok(KeyCode::KeyI),
				Ok(KeyCode::KeyK),
			],
			GameState::iterator()
				.map(KeyCode::try_from)
				.collect::<Vec<_>>()
		)
	}
}
