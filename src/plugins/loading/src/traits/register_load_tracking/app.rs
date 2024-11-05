use super::{Loaded, RegisterLoadTracking, RegisterLoadTrackingInSubApp};
use crate::{
	resources::track::Track,
	systems::is_processing::is_processing,
	traits::progress::Progress,
};
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::*};

impl<TMarker> RegisterLoadTracking<TMarker> for App {
	fn register_load_tracking<T, TProgress>(
		&mut self,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static,
	{
		self.add_systems(
			Update,
			system
				.pipe(Track::<TProgress>::track::<T>)
				.run_if(is_processing::<TProgress>),
		)
	}
}

impl<TMarker> RegisterLoadTrackingInSubApp<TMarker> for App {
	fn register_load_tracking_in_sub_app<T, TProgress>(
		&mut self,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static,
	{
		self.sub_app_mut(app_label).add_systems(
			schedule,
			system
				.pipe(Track::<TProgress>::track_in_main_world::<T>)
				.run_if(Track::<TProgress>::main_world_is_processing),
		);

		self
	}
}
