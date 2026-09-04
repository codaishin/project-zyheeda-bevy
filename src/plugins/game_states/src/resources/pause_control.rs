use bevy::prelude::*;
use common::traits::handles_game_states::{GameState, Gui};
use macros::EnumConversions;
use std::{collections::HashSet, hash::Hash};

#[derive(Resource, Debug, PartialEq, Default, Clone)]
pub(crate) struct PauseControl {
	pub(crate) non_pause_states: HashSet<Pausable>,
}

impl PauseControl {
	pub(crate) const DEFAULT_NON_PAUSE: GameState = GameState::Play;

	pub(crate) fn is_pause_state(&self, state: impl Into<Pausable>) -> bool {
		let state = state.into();

		if state == Self::DEFAULT_NON_PAUSE {
			return false;
		}

		!self.non_pause_states.contains(&state)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum Pausable {
	GameState(GameState),
	Gui(Gui),
}
