mod app;

use super::progress::Progress;
use crate::resources::track::Loaded;
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::IntoSystem};

pub trait RegisterLoadTracking<TMarker> {
	fn register_load_tracking<T, TProgress>(
		&mut self,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static;
}

pub trait RegisterLoadTrackingInSubApp<TMarker> {
	fn register_load_tracking_in_sub_app<T, TProgress>(
		&mut self,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), Loaded, TMarker>,
	) -> &mut Self
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static;
}
