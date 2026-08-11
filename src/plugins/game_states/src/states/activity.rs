use bevy::prelude::*;
use common::traits::handles_game_states::{ActivityState, GameState, ReadState};

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
	fn from(activity: ActivityState) -> Self {
		match activity {
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
	fn from(read: ReadState) -> Self {
		match read {
			ReadState::Loading => Self::Loading,
		}
	}
}

impl From<&Activity> for GameState {
	fn from(activity: &Activity) -> Self {
		use ActivityState::*;
		use ReadState::*;

		match activity {
			Activity::LoadingEssentialAssets => Self::Activity(LoadingEssentialAssets),
			Activity::LoadDependencies => Self::Activity(LoadDependencies),
			Activity::StartScreen => Self::Activity(StartScreen),
			Activity::NewGame => Self::Activity(NewGame),
			Activity::Play => Self::Activity(Play),
			Activity::Paused => Self::Activity(Paused),
			Activity::Save => Self::Activity(Save),
			Activity::Load => Self::Activity(Load),
			Activity::Loading => Self::Read(Loading),
		}
	}
}
