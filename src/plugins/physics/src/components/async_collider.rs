use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AsyncConvexCollider {
	pub(crate) path: &'static str,
	pub(crate) mesh: Option<Handle<Mesh>>,
	pub(crate) scale: Option<ColliderScale>,
}
