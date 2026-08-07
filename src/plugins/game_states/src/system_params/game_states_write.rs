use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{GameState, GameStates, GameStatesMut};
use zyheeda_core::collections::ordered::OrderedSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w> {
	ctx: ResMut<'w, GameStatesContext>,
}

impl GameStates for GameStatesWrite<'_> {
	fn game_states(&self) -> &OrderedSet<GameState> {
		&self.ctx.states
	}
}

impl GameStatesMut for GameStatesWrite<'_> {
	fn game_states_mut(&mut self) -> &mut OrderedSet<GameState> {
		&mut self.ctx.states
	}
}
