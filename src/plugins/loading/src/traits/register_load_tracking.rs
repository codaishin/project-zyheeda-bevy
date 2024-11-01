mod app;

use crate::resources::load_tracker::Loaded;
use bevy::prelude::IntoSystem;

pub trait RegisterLoadTracking<TMarker> {
	fn register_load_tracking<T, TSystem>(&mut self, system: TSystem) -> &mut Self
	where
		T: 'static,
		TSystem: IntoSystem<(), Loaded, TMarker>;
}
