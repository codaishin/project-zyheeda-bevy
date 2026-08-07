use bevy::prelude::*;
use common::traits::handles_game_states::{ActivityState, GameState};

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct GameStateInternal(pub(crate) GameState);

impl Default for GameStateInternal {
	fn default() -> Self {
		Self(GameState::Activity(ActivityState::LoadingEssentialAssets))
	}
}
