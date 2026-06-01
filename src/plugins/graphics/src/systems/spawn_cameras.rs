use crate::components::camera_labels::{CompositePass, OutlinePass, Ui, WorldPass};
use bevy::prelude::*;

pub(crate) fn spawn_cameras(mut commands: Commands) {
	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("World Camera"),
		WorldPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("Outline Camera"),
		OutlinePass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("Composite Camera"),
		CompositePass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("UI Camera"),
		Ui,
	));
}
