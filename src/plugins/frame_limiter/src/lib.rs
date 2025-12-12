#[cfg(debug_assertions)]
mod debug;

use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSet},
};
use std::{
	thread,
	time::{Duration, Instant},
};

/// A plugin inspired by the `bevy_framepace` plugin:
/// <https://github.com/aevyrie/bevy_framepace>.
///
/// This plugin implements a stripped down frame-limiting logic,
/// designed to cap the frames per second (FPS) at the specified
/// `target_fps`. Its primary purpose is to mitigate unexpected FPS
/// drops that can occur on certain systems (e.g., Linux with X11
/// and Nvidia GPUs) during mouse movement or clicks.
///
/// The frame rate can only be limited within the range of 1 to 60 FPS
/// due to the render schedule (which we hook into) running at 60 FPS.
pub struct FrameLimiterPlugin {
	pub target_fps: u32,
}

impl Plugin for FrameLimiterPlugin {
	fn build(&self, app: &mut App) {
		#[cfg(debug_assertions)]
		debug::init(app);

		let time_per_frame = match self.target_fps {
			0 => {
				error!("Target FPS was set to 0, using 1 FPS instead");
				Duration::from_secs(1)
			}
			target_fps if target_fps > 60 => {
				error!("Target FPS was set to >60, using 60 FPS instead");
				Duration::from_secs(1) / 60
			}
			target_fps => Duration::from_secs(1) / target_fps,
		};

		app.insert_resource(Time::<Fixed>::from_duration(time_per_frame));

		let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
			error!(
				"RenderApp is unavailable. Frame limiting will not apply to rendering, only to fixed schedules."
			);
			return;
		};

		render_app
			.insert_resource(Sleep(time_per_frame))
			.insert_resource(LastSleep(Instant::now()))
			.add_systems(Render, Sleep::system.in_set(RenderSet::Cleanup));
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
