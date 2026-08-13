use crate::resources::game_state_context::GameStatesContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{GameStateCollection, GameStates};

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	ctx: Res<'w, GameStatesContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn game_states(&self) -> GameStateCollection<'_> {
		GameStateCollection {
			activity: self.ctx.activity,
			ui: &self.ctx.ui,
		}
	}
}
