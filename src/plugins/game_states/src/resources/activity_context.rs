use crate::states::activity::Activity;
use bevy::prelude::*;
use common::traits::handles_game_states::ActivityState;

#[derive(Resource)]
pub(crate) struct ActivityContext {
	pub(crate) activity: ActivityState,
}

impl FromWorld for ActivityContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			activity: ActivityState::from(world.resource::<State<Activity>>().get()),
		}
	}
}
