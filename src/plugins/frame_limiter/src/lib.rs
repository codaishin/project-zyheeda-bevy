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
		{
			use debug::*;

			app.insert_resource(Fps(u32::MAX))
				.add_systems(Startup, DisplayFps::spawn)
				.add_systems(Update, (MeasureFps::system, DisplayFps::update).chain());
		}
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
	use super::*;
	use std::{ops::DerefMut, time::Instant};

	#[derive(Resource, Debug, PartialEq)]
	pub(super) struct Fps(pub(super) u32);

	#[derive(Debug, PartialEq)]
	pub(super) struct MeasureFps {
		timer: Instant,
		counter: u32,
	}

	impl Default for MeasureFps {
		fn default() -> Self {
			Self {
				timer: Instant::now(),
				counter: 0,
			}
		}
	}

	impl MeasureFps {
		pub(super) fn system(mut measure: Local<MeasureFps>, mut fps: ResMut<Fps>) {
			let measure = measure.deref_mut();
			measure.counter += 1;

			if measure.timer.elapsed().as_secs() < 1 {
				return;
			}

			*fps = Fps(measure.counter);
			*measure = MeasureFps::default();
		}
	}

	#[derive(Component, Debug, PartialEq)]
	#[require(Node(DisplayFps::top_left), Text)]
	pub(super) struct DisplayFps;

	impl DisplayFps {
		fn top_left() -> Node {
			Node {
				position_type: PositionType::Absolute,
				left: Val::Px(10.),
				top: Val::Px(10.),
				width: Val::Px(100.),
				height: Val::Px(20.),
				..default()
			}
		}

		pub(super) fn spawn(mut commands: Commands) {
			commands.spawn(DisplayFps);
		}

		pub(super) fn update(mut displays: Query<&mut Text, With<DisplayFps>>, fps: Res<Fps>) {
			if !fps.is_changed() {
				return;
			}

			let Ok(mut text) = displays.get_single_mut() else {
				return;
			};

			let Fps(fps) = *fps;
			*text = Text(format!("FPS: {fps}"));
		}
	}
}
