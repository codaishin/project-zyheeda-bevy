use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{GameState, GameStates};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	ctx: Res<'w, GameStatesContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn game_states(&self) -> &HashSet<GameState> {
		&self.ctx.states
	}
}
