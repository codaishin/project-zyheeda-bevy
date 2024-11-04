use super::{Loaded, RegisterLoadTracking, RegisterLoadTrackingInSubApp};
use crate::{resources::load_tracker::LoadTracker, systems::is_loading::is_loading};
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::*};

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

impl<TMarker> RegisterLoadTrackingInSubApp<TMarker> for App {
	fn register_load_tracking_in_sub_app<T>(
		&mut self,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
	{
		self.sub_app_mut(app_label).add_systems(
			schedule,
			system
				.pipe(LoadTracker::track_in_main_world::<T>)
				.run_if(LoadTracker::main_world_is_loading),
		);

		self
	}
}
