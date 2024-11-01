mod app;

use crate::resources::load_tracker::Loaded;
use bevy::prelude::IntoSystem;

pub trait RegisterLoadTracking<TMarker> {
	fn register_load_tracking<T>(
		&mut self,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static;
}
