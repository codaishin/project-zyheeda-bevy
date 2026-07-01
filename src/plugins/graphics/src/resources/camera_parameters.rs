use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct CameraParameters {
	pub(crate) ui_cam: Option<Entity>,
	pub(crate) transform: Transform,
	pub(crate) uis: Vec<Entity>,
}
