use super::{load_state::LoadState, menu_state::MenuState};
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
	Loading(LoadState),
	NewGame,
	Play,
	IngameMenu(MenuState),
}

#[derive(Debug, PartialEq)]
pub struct NoKeySet;

impl TryFrom<GameState> for KeyCode {
	type Error = NoKeySet;

	fn try_from(value: GameState) -> Result<Self, Self::Error> {
		let GameState::IngameMenu(menu) = value else {
			return Err(NoKeySet);
		};

		Ok(match menu {
			MenuState::Inventory => KeyCode::KeyI,
			MenuState::ComboOverview => KeyCode::KeyK,
		})
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
			GameState::NewGame => Some(GameState::Loading(LoadState::Assets)),
			GameState::Loading(LoadState::Assets) => {
				Some(GameState::Loading(LoadState::Dependencies))
			}
			GameState::Loading(LoadState::Dependencies) => Some(GameState::Play),
			GameState::Play => Some(GameState::IngameMenu(MenuState::Inventory)),
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
				GameState::Loading(LoadState::Assets),
				GameState::Loading(LoadState::Dependencies),
				GameState::Play,
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
				Err(NoKeySet),
				Ok(KeyCode::KeyI),
				Ok(KeyCode::KeyK)
			],
			GameState::iterator()
				.map(KeyCode::try_from)
				.collect::<Vec<_>>()
		)
	}
}
