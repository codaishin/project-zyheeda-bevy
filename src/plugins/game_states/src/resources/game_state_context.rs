use bevy::prelude::*;
use common::traits::handles_game_states::GameState;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(crate) struct GameStatesContext {
	pub(crate) states: HashSet<GameState>,
}
