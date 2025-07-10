use super::menu_state::MenuState;
use crate::{
	states::save_state::SaveState,
	tools::iter_helpers::{first, next},
	traits::{
		automatic_transitions::{AutoTransitions, TransitionTo},
		handles_load_tracking::LoadGroup,
		iteration::{Iter, IterFinite},
		pause_control::PauseControl,
		states::PlayState,
	},
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
	LoadDependencies,
	NewGame,
	Play,
	Save(SaveState),
	IngameMenu(MenuState),
}

impl AutoTransitions for GameState {
	fn auto_transitions() -> impl IntoIterator<Item = (Self, TransitionTo<Self>)> {
		const {
			[
				(Self::NewGame, TransitionTo::State(Self::LoadDependencies)),
				(
					Self::Save(SaveState::Load),
					TransitionTo::State(Self::LoadDependencies),
				),
				(Self::Save(SaveState::Save), TransitionTo::PreviousState),
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
			GameState::NewGame => Some(GameState::LoadDependencies),
			GameState::LoadDependencies => Some(GameState::Play),
			GameState::Play => first(GameState::Save),
			GameState::Save(s) => next(GameState::Save, *s).or(first(GameState::IngameMenu)),
			GameState::IngameMenu(s) => next(GameState::IngameMenu, *s),
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

	const LOAD_STATE: GameState = GameState::LoadDependencies;
	const LOAD_DONE_STATE: GameState = GameState::Play;

	fn load_reset_states() -> Vec<Self::TState> {
		vec![GameState::NewGame, GameState::Save(SaveState::Load)]
	}
}

#[derive(Debug, PartialEq)]
pub struct NoKeySet;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_all_states() {
		let game_states = std::iter::empty()
			.chain([
				GameState::LoadingEssentialAssets,
				GameState::StartMenu,
				GameState::NewGame,
				GameState::LoadDependencies,
				GameState::Play,
			])
			.chain(SaveState::iterator().map(GameState::Save))
			.chain(MenuState::iterator().map(GameState::IngameMenu))
			.collect::<Vec<_>>();

		assert_eq!(
			game_states,
			GameState::iterator().take(100).collect::<Vec<_>>(),
		)
	}
}
