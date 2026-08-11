use bevy::prelude::*;
use common::traits::handles_game_states::{ActivityState, ReadState};

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub(crate) enum Activity {
	#[default]
	LoadingEssentialAssets,
	LoadDependencies,
	StartScreen,
	NewGame,
	Play,
	Paused,
	Save,
	Load,
	Loading,
}

impl From<ActivityState> for Activity {
	fn from(value: ActivityState) -> Self {
		match value {
			ActivityState::LoadingEssentialAssets => Self::LoadingEssentialAssets,
			ActivityState::LoadDependencies => Self::LoadDependencies,
			ActivityState::StartScreen => Self::StartScreen,
			ActivityState::NewGame => Self::NewGame,
			ActivityState::Play => Self::Play,
			ActivityState::Paused => Self::Paused,
			ActivityState::Save => Self::Save,
			ActivityState::Load => Self::Load,
		}
	}
}

impl From<ReadState> for Activity {
	fn from(value: ReadState) -> Self {
		match value {
			ReadState::Loading => Self::Loading,
		}
	}
}
