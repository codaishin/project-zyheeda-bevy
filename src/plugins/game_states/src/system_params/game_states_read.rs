use crate::resources::game_state_context::GameStateContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesReadParam<'w> {
	current: Res<'w, GameStateContext>,
}

impl GameStates for GameStatesReadParam<'_> {
	fn command(&self) -> Option<GameStateCommand> {
		self.current.command_state.try_into().ok()
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		&self.current.ui
	}
}
