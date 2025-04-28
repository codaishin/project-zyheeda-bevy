use super::menu_state::MenuState;
use crate::traits::{
	handles_load_tracking::LoadGroup,
	iteration::{Iter, IterFinite},
	states::PlayState,
};
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
	#[default]
	None,
	LoadingEssentialAssets,
	StartMenu,
	Loading,
	NewGame,
	Play,
	Saving,
	IngameMenu(MenuState),
}

pub struct LoadingEssentialAssets;

impl LoadGroup for LoadingEssentialAssets {
	type TState = GameState;

	const LOAD_STATE: GameState = GameState::LoadingEssentialAssets;
	const LOAD_DONE_STATE: GameState = GameState::StartMenu;
}

pub struct LoadingGame;

impl LoadGroup for LoadingGame {
	type TState = GameState;

	const LOAD_STATE: GameState = GameState::Loading;
	const LOAD_DONE_STATE: GameState = GameState::Play;
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
			GameState::IngameMenu(MenuState::Setup) => Ok(KeyCode::Escape),
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
			GameState::None => Some(GameState::LoadingEssentialAssets),
			GameState::LoadingEssentialAssets => Some(GameState::StartMenu),
			GameState::StartMenu => Some(GameState::NewGame),
			GameState::NewGame => Some(GameState::Loading),
			GameState::Loading => Some(GameState::Play),
			GameState::Play => Some(GameState::Saving),
			GameState::Saving => Some(GameState::IngameMenu(MenuState::Inventory)),
			GameState::IngameMenu(MenuState::Inventory) => {
				Some(GameState::IngameMenu(MenuState::ComboOverview))
			}
			GameState::IngameMenu(MenuState::ComboOverview) => {
				Some(GameState::IngameMenu(MenuState::Setup))
			}
			GameState::IngameMenu(MenuState::Setup) => None,
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
				GameState::LoadingEssentialAssets,
				GameState::StartMenu,
				GameState::NewGame,
				GameState::Loading,
				GameState::Play,
				GameState::Saving,
				GameState::IngameMenu(MenuState::Inventory),
				GameState::IngameMenu(MenuState::ComboOverview),
				GameState::IngameMenu(MenuState::Setup),
			],
			GameState::iterator().take(100).collect::<Vec<_>>(),
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
				Err(NoKeySet),
				Ok(KeyCode::F5),
				Ok(KeyCode::KeyI),
				Ok(KeyCode::KeyK),
				Ok(KeyCode::Escape),
			],
			GameState::iterator()
				.take(100)
				.map(KeyCode::try_from)
				.collect::<Vec<_>>()
		)
	}
}
