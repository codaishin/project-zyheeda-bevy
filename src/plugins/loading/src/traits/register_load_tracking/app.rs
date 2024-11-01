use super::{Loaded, RegisterLoadTracking};
use crate::resources::load_tracker::LoadTracker;
use bevy::prelude::*;

impl<TMarker> RegisterLoadTracking<TMarker> for App {
	fn register_load_tracking<T, TSystem>(&mut self, system: TSystem) -> &mut Self
	where
		T: 'static,
		TSystem: IntoSystem<(), Loaded, TMarker>,
	{
		self.add_systems(
			Update,
			system
				.pipe(LoadTracker::track::<T>)
				.run_if(resource_exists::<LoadTracker>),
		)
	}
}
