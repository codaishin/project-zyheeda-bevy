use bevy::prelude::*;
use common::traits::handles_game_states::GameState;
use zyheeda_core::collections::ordered::OrderedSet;

#[derive(Resource, Default)]
pub(crate) struct GameStatesContext {
	pub(crate) states: OrderedSet<GameState>,
}
