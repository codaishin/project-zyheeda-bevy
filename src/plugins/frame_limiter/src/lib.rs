use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSet},
};
use std::{
	thread,
	time::Duration,
};

pub struct FrameLimiterPlugin;

impl Plugin for FrameLimiterPlugin {
	fn build(&self, app: &mut App) {
		app.sub_app_mut(RenderApp)
			.insert_resource(Sleep(Duration::from_secs(1) / 60))
			.add_systems(Render, Sleep::system.in_set(RenderSet::Cleanup));
	}
}

#[derive(Resource, Debug, PartialEq)]
struct Sleep(Duration);

impl Sleep {
	fn system(sleep: Res<Sleep>) {
		let Sleep(sleep) = *sleep;

		thread::sleep(sleep);
	}
}
