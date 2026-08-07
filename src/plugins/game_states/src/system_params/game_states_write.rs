use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{
	AddGameState,
	GameState,
	GameStates,
	IntoGameStateAdd,
	IntoGameStateRemove,
	RemoveGameState,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w> {
	ctx: ResMut<'w, GameStatesContext>,
}

impl GameStates for GameStatesWrite<'_> {
	fn game_states(&self) -> &HashSet<GameState> {
		&self.ctx.states
	}
}

impl AddGameState for GameStatesWrite<'_> {
	fn add_game_state<T>(&mut self, _: T)
	where
		T: IntoGameStateAdd,
	{
	}
}

impl RemoveGameState for GameStatesWrite<'_> {
	fn remove_game_state<T>(&mut self, _: &T)
	where
		T: IntoGameStateRemove,
	{
	}
}
