use crate::components::camera_labels::{
	AgentsPass,
	CompositePass,
	OutlinePass,
	UiPass,
	VisibilityPass,
	WorldLight,
	WorldPass,
};
use bevy::prelude::*;

pub(crate) fn spawn_cameras(mut commands: Commands) {
	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("World Camera"),
		WorldPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("Agents Camera"),
		AgentsPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("Visibility Camera"),
		VisibilityPass,
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
		UiPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("World Light"),
		WorldLight,
	));
}
