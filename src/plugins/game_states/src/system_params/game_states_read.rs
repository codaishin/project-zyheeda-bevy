use crate::resources::game_state_context::GameStateContext;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	current: Res<'w, GameStateContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn activity(&self) -> Activity {
		self.current.activity
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		&self.current.ui
	}
}
