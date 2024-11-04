mod app;

use crate::resources::load_tracker::Loaded;
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::IntoSystem};

pub trait RegisterLoadTracking<TMarker> {
	fn register_load_tracking<T>(
		&mut self,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static;
}

pub trait RegisterLoadTrackingInSubApp<TMarker> {
	fn register_load_tracking_in_sub_app<T>(
		&mut self,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static;
}
