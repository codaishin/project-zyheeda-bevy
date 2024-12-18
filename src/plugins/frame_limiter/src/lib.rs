use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSet},
};
use std::{
	thread,
	time::{Duration, Instant},
};

pub struct FrameLimiterPlugin;

impl Plugin for FrameLimiterPlugin {
	fn build(&self, app: &mut App) {
		app.sub_app_mut(RenderApp)
			.insert_resource(LastSleep(Instant::now()))
			.insert_resource(Sleep(Duration::from_secs(1) / 60))
			.add_systems(Render, Sleep::system.in_set(RenderSet::Cleanup));

		#[cfg(debug_assertions)]
		app.add_systems(Update, debug::MeasureFps::system);
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

#[cfg(debug_assertions)]
mod debug {
	use bevy::prelude::Local;
	use std::{ops::DerefMut, time::Instant};

	#[derive(Debug, PartialEq)]
	pub(super) struct MeasureFps {
		timer: Instant,
		counter: u32,
		fps: u32,
	}

	impl Default for MeasureFps {
		fn default() -> Self {
			Self {
				timer: Instant::now(),
				counter: 0,
				fps: u32::MAX,
			}
		}
	}

	impl MeasureFps {
		pub(super) fn system(mut fps: Local<MeasureFps>) {
			let fps = fps.deref_mut();
			fps.counter += 1;

			if fps.timer.elapsed().as_secs() < 1 {
				return;
			}

			fps.fps = fps.counter;
			fps.timer = Instant::now();
			fps.counter = 0;
			println!("FPS: {}", fps.fps);
		}
	}
}
