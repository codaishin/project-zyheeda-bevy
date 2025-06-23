use crate::components::camera_labels::{FirstPass, SecondPass, Ui};
use bevy::prelude::*;

pub(crate) fn spawn_cameras(mut commands: Commands) {
	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("First Pass Camera"),
		FirstPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("Second Pass Camera"),
		SecondPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("UI Camera"),
		Ui,
	));
}
