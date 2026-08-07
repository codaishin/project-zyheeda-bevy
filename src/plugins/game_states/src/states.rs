use bevy::prelude::*;
use common::traits::handles_game_states::{ActivityState, MenuState, SaveState};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, States)]
pub(crate) enum GameState {
	Activity(ActivityState),
	Save(SaveState),
	Menu(MenuState),
}

impl Default for GameState {
	fn default() -> Self {
		Self::Activity(ActivityState::LoadingEssentialAssets)
	}
}

impl From<ActivityState> for GameState {
	fn from(state: ActivityState) -> Self {
		Self::Activity(state)
	}
}

impl From<SaveState> for GameState {
	fn from(state: SaveState) -> Self {
		Self::Save(state)
	}
}

impl From<MenuState> for GameState {
	fn from(state: MenuState) -> Self {
		Self::Menu(state)
	}
}
