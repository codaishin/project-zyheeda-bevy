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
