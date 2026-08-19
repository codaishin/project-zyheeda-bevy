use crate::GameStatesPlugin;
use bevy::prelude::*;
use common::prelude::*;
use std::{collections::HashSet, sync::LazyLock};

#[derive(Resource, Debug, PartialEq, Clone)]
pub(crate) struct GameStateRoles {
	pub(crate) non_pause_states: HashSet<GameState>,
}

pub(crate) static GAME_STATE_ROLES_DEFAULT: LazyLock<GameStateRoles> =
	LazyLock::new(|| GameStateRoles {
		non_pause_states: HashSet::from_iter(
			GameStatesPlugin::DEFAULT
				.iter()
				.copied()
				.map(GameState::Activity),
		),
	});

impl GameStateRoles {
	pub(crate) fn is_pause_state(&self, state: impl Into<GameState>) -> bool {
		!self.non_pause_states.contains(&state.into())
	}
}

impl Default for GameStateRoles {
	fn default() -> Self {
		GAME_STATE_ROLES_DEFAULT.clone()
	}
}
