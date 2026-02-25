use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AsyncCollider {
	pub(crate) path: &'static str,
	pub(crate) mesh: Option<Handle<Mesh>>,
	pub(crate) scale: Option<ColliderScale>,
	pub(crate) collider_type: ColliderType,
}

impl AsyncCollider {
	pub(crate) const fn concave(path: &'static str) -> Self {
		Self {
			path,
			mesh: None,
			scale: None,
			collider_type: ColliderType::Concave,
		}
	}

	pub(crate) const fn convex(path: &'static str) -> Self {
		Self {
			path,
			mesh: None,
			scale: None,
			collider_type: ColliderType::Convex,
		}
	}

	pub(crate) fn with_scale(mut self, scale: ColliderScale) -> Self {
		self.scale = Some(scale);
		self
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum ColliderType {
	Convex,
	Concave,
}
