use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AsyncCollider {
	pub(crate) source: Source,
	pub(crate) scale: Option<ColliderScale>,
	pub(crate) collider_type: ColliderType,
}

#[cfg(test)]
impl AsyncCollider {
	pub(crate) fn concave(source: impl Into<Source>) -> Self {
		Self {
			source: source.into(),
			scale: None,
			collider_type: ColliderType::Concave,
		}
	}

	pub(crate) fn convex(source: impl Into<Source>) -> Self {
		Self {
			source: source.into(),
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
pub(crate) enum Source {
	Path(&'static str),
	MeshOfEntity,
	Handle(Handle<Mesh>),
}

impl From<&'static str> for Source {
	fn from(path: &'static str) -> Self {
		Self::Path(path)
	}
}

impl From<Handle<Mesh>> for Source {
	fn from(handle: Handle<Mesh>) -> Self {
		Self::Handle(handle)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ColliderType {
	/// All line segments between any two points on the surface remain inside or on the mesh
	Convex,
	/// Some line segments between two points on the surface do not remain inside or on the mesh
	Concave,
}
