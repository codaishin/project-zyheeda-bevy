use bevy::prelude::*;
use common::traits::handles_game_states::ActivityState;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct Activity(pub(crate) ActivityState);

impl Default for Activity {
	fn default() -> Self {
		Self(ActivityState::LoadingEssentialAssets)
	}
}

impl From<ActivityState> for Activity {
	fn from(activity: ActivityState) -> Self {
		Self(activity)
	}
}

impl From<&Activity> for ActivityState {
	fn from(Activity(activity): &Activity) -> Self {
		*activity
	}
}
