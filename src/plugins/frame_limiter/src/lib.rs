use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSystems},
};
use std::{
	thread,
	time::{Duration, Instant},
};
use zyheeda_core::prelude::*;

/// A plugin inspired by the `bevy_framepace` plugin:
/// <https://github.com/aevyrie/bevy_framepace>.
///
/// This plugin implements a stripped down frame-limiting logic,
/// designed to cap the frames per second (FPS) at the specified
/// `target_fps`. Its primary purpose is to mitigate unexpected FPS
/// drops that can occur on certain systems (e.g., Linux with X11
/// and Nvidia GPUs) during mouse movement or clicks.
pub struct FrameLimiterPlugin {
	pub target_fps: DisplayFPS,
}

impl Plugin for FrameLimiterPlugin {
	fn build(&self, app: &mut App) {
		let time_per_frame = Duration::from_secs(1) / (*self.target_fps).into();

		app.sub_app_mut(RenderApp)
			.insert_resource(Sleep(time_per_frame))
			.insert_resource(LastSleep(Instant::now()))
			.add_systems(Render, Sleep::system.in_set(RenderSystems::Cleanup));
	}
}

#[derive(Resource, Debug, PartialEq)]
struct LastSleep(Instant);

#[derive(Resource, Debug, PartialEq)]
struct Sleep(Duration);

impl Sleep {
	fn system(sleep: Res<Sleep>, mut last_sleep: ResMut<LastSleep>) {
		let Sleep(sleep) = *sleep;
		let LastSleep(last_sleep) = last_sleep.as_mut();
		let sleep = sleep.saturating_sub(last_sleep.elapsed());

		thread::sleep(sleep);
		*last_sleep = Instant::now();
	}
}
