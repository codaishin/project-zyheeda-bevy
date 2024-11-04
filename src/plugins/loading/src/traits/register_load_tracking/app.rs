use super::{Loaded, RegisterLoadTracking};
use crate::{resources::load_tracker::LoadTracker, systems::is_loading::is_loading};
use bevy::prelude::*;

impl<TMarker> RegisterLoadTracking<TMarker> for App {
	fn register_load_tracking<T>(
		&mut self,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
	{
		self.add_systems(
			Update,
			system.pipe(LoadTracker::track::<T>).run_if(is_loading),
		)
	}
}
