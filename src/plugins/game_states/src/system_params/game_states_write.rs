use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	handles_game_states::{GameStates, GameStatesMut},
	thread_safe::ThreadSafe,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w, T>
where
	T: ThreadSafe,
{
	ctx: ResMut<'w, GameStatesContext<T>>,
}

impl<T> GameStates<T> for GameStatesWrite<'_, T>
where
	T: ThreadSafe,
{
	fn game_states(&self) -> &HashSet<T> {
		&self.ctx.states
	}
}

impl<T> GameStatesMut<T> for GameStatesWrite<'_, T>
where
	T: ThreadSafe,
{
	fn game_states_mut(&mut self) -> &mut HashSet<T> {
		&mut self.ctx.states
	}
}
