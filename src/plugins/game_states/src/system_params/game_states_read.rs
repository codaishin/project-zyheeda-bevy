use crate::resources::game_state_context::GameStateContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{GameStateCollection, GameStates};

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	current: Res<'w, GameStateContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn game_states(&self) -> GameStateCollection<'_> {
		GameStateCollection {
			activity: self.current.activity,
			ui: &self.current.ui,
		}
	}
}
