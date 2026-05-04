use crate::errors::{ErrorData, Level};
use bevy::prelude::*;
use std::fmt::Display;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) enum LoadModel {
	Scene(Handle<Scene>),
	GltfError(GltfSceneError),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct GltfSceneError {
	pub(crate) scene_count: usize,
	pub(crate) requested_id: usize,
}

impl Display for GltfSceneError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cannot process scene with id {} (scene count: {})",
			self.requested_id, self.scene_count
		)
	}
}

impl ErrorData for GltfSceneError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Gltf Scene Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}
