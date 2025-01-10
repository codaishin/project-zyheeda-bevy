use crate::{
	components::camera_labels::{FirstPass, FirstPassTexture, SecondPass, Ui},
	resources::first_pass_image::FirstPassImage,
};
use bevy::prelude::*;

pub(crate) fn spawn_cameras(In(first_pass_image): In<FirstPassImage>, mut commands: Commands) {
	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("First Pass Camera"),
		FirstPass,
	));

	commands.spawn((
		#[cfg(debug_assertions)]
		Name::from("First Pass Texture Camera"),
		FirstPassTexture::from_image(first_pass_image.handle.clone()),
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

	commands.insert_resource(first_pass_image);
}
