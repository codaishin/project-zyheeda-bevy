use bevy::prelude::*;
use common::traits::handles_physics::physical_bodies::{Blocker, Body, PhysicsType, Shape};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct MeshCollider;

impl MeshCollider {
	pub(crate) fn body<TBody>() -> TBody
	where
		TBody: From<Body>,
	{
		TBody::from(
			Body::from_shape(Shape::StaticGltfMesh3d)
				.with_physics_type(PhysicsType::Terrain)
				.with_blocker_types(Blocker::Physical),
		)
	}
}
