use bevy::prelude::*;
use common::traits::handles_load_tracking::{IsProcessing, Progress};

#[derive(States, Debug, PartialEq, Clone, Copy, Eq, Hash, Default)]
pub(crate) enum LoadState {
	#[default]
	Idle,
	LoadAssets,
	ResolveDependencies,
	Done,
}

impl LoadState {
	pub(crate) fn processing<TProgress>() -> Self
	where
		TProgress: Progress,
	{
		match TProgress::IS_PROCESSING {
			IsProcessing::Assets => LoadState::LoadAssets,
			IsProcessing::Dependencies => LoadState::ResolveDependencies,
		}
	}
}
