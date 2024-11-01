pub mod traits;

mod resources;

use bevy::prelude::*;
use common::traits::{init_resource::InitResource, remove_resource::RemoveResource};
use resources::load_tracker::LoadTracker;

pub struct LoadingPlugin<TState> {
	pub load_state: TState,
}

impl<TState> Plugin for LoadingPlugin<TState>
where
	TState: States + Copy,
{
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(self.load_state), LoadTracker::init)
			.add_systems(OnExit(self.load_state), LoadTracker::remove);
	}
}
