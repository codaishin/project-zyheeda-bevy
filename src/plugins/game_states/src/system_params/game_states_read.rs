use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{handles_game_states::GameStates, thread_safe::ThreadSafe};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesRead<'w, T>
where
	T: ThreadSafe,
{
	ctx: ResMut<'w, GameStatesContext<T>>,
}

impl<T> GameStates<T> for GameStatesRead<'_, T>
where
	T: ThreadSafe,
{
	fn game_states(&self) -> &HashSet<T> {
		&self.ctx.states
	}
}
