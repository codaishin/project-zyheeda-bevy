use crate::components::main_camera::MainCamera;
use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
};

pub(crate) fn spawn_camera(mut commands: Commands) {
	commands.spawn((
		MainCamera,
		Camera3d::default(),
		Camera {
			hdr: true,
			..default()
		},
		Tonemapping::TonyMcMapface,
		Bloom::default(),
	));
}
