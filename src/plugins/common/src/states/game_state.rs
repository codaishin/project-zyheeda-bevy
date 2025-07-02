use super::menu_state::MenuState;
use crate::traits::{
	automatic_transitions::{AutoTransitions, TransitionTo},
	handles_load_tracking::LoadGroup,
	iteration::{Iter, IterFinite},
	pause_control::PauseControl,
	states::PlayState,
};
use bevy::prelude::*;

/// Main state to represent the state of the game.
///
/// Should be used to control scheduling of state dependent systems like:
/// - UI
/// - AI
/// - physics
/// - saving/loading
/// - ...
///
/// Various functionalities, including pausing and automatic state transitions
/// require:
/// - either [`CommonPlugin`](crate::CommonPlugin): `app.add_plugins(CommonPlugin)`
/// - or [`RegisterControlledState`](crate::traits::register_controlled_state::RegisterControlledState):
///   `app.register_controlled_state::<GameState>()`
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
	#[default]
	LoadingEssentialAssets,
	StartMenu,
	Loading,
	NewGame,
	Play,
	Saving,
	LoadingSave,
	IngameMenu(MenuState),
}

impl AutoTransitions for GameState {
	fn auto_transitions() -> impl IntoIterator<Item = (Self, TransitionTo<Self>)> {
		const {
			[
				(Self::NewGame, TransitionTo::State(Self::Loading)),
				(Self::Saving, TransitionTo::PreviousState),
				(Self::LoadingSave, TransitionTo::State(Self::Play)),
			]
		}
	}
}

impl PauseControl for GameState {
	fn non_pause_states() -> impl IntoIterator<Item = Self> {
		const { [Self::Play] }
	}
}

impl IterFinite for GameState {
	fn iterator() -> Iter<Self> {
		Iter(Some(GameState::LoadingEssentialAssets))
	}

	fn next(Iter(current): &Iter<Self>) -> Option<Self> {
		match current.as_ref()? {
			GameState::LoadingEssentialAssets => Some(GameState::StartMenu),
			GameState::StartMenu => Some(GameState::NewGame),
			GameState::NewGame => Some(GameState::Loading),
			GameState::Loading => Some(GameState::Play),
			GameState::Play => Some(GameState::Saving),
			GameState::Saving => Some(GameState::LoadingSave),
			GameState::LoadingSave => Some(GameState::IngameMenu(MenuState::Inventory)),
			GameState::IngameMenu(MenuState::Inventory) => {
				Some(GameState::IngameMenu(MenuState::ComboOverview))
			}
			GameState::IngameMenu(MenuState::ComboOverview) => {
				Some(GameState::IngameMenu(MenuState::Settings))
			}
			GameState::IngameMenu(MenuState::Settings) => None,
		}
	}
}

impl PlayState for GameState {
	fn play_state() -> Self {
		Self::Play
	}
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

	fn load_reset_states() -> Vec<Self::TState> {
		vec![GameState::NewGame, GameState::LoadingSave]
	}
}

#[derive(Debug, PartialEq)]
pub struct NoKeySet;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_all_states() {
		assert_eq!(
			vec![
				GameState::LoadingEssentialAssets,
				GameState::StartMenu,
				GameState::NewGame,
				GameState::Loading,
				GameState::Play,
				GameState::Saving,
				GameState::LoadingSave,
				GameState::IngameMenu(MenuState::Inventory),
				GameState::IngameMenu(MenuState::ComboOverview),
				GameState::IngameMenu(MenuState::Settings),
			],
			GameState::iterator().take(100).collect::<Vec<_>>(),
		)
	}
}
