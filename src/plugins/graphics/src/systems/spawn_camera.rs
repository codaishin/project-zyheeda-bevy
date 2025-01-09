use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
};

use crate::components::main_camera::MainCamera;

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
