use crate::GameStatesPlugin;
use bevy::prelude::*;
use common::traits::handles_game_states::{GameState, NonPausedStates};
use std::collections::HashSet;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct GameStateRoles {
	pub(crate) non_pause_states: HashSet<GameState>,
}

impl GameStateRoles {
	pub(crate) fn is_pause_state(&self, state: impl Into<GameState>) -> bool {
		!self.non_pause_states.contains(&state.into())
	}
}

impl Default for GameStateRoles {
	fn default() -> Self {
		Self {
			non_pause_states: HashSet::from_iter(
				GameStatesPlugin::DEFAULT
					.iter()
					.copied()
					.map(GameState::Activity),
			),
		}
	}
}
