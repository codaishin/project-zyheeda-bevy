use bevy::prelude::*;
use common::prelude::*;

#[derive(States, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) struct ActivityState(pub(crate) Activity);

impl From<Activity> for ActivityState {
	fn from(activity: Activity) -> Self {
		Self(activity)
	}
}

impl From<&ActivityState> for Activity {
	fn from(ActivityState(activity): &ActivityState) -> Self {
		*activity
	}
}
