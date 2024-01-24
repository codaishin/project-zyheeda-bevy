pub mod dummy;
pub mod projectile;

use bevy::{asset::Asset, math::Vec3, render::mesh::Mesh};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};

pub trait Model<TMaterial: Asset> {
	fn material() -> TMaterial;
	fn mesh() -> Mesh;
}

pub trait Shape<TShape> {
	fn shape() -> TShape;
}

pub trait Offset {
	fn offset() -> Vec3;
}

pub trait GetCollider {
	fn collider() -> Collider;
}

pub trait GetRigidBody {
	fn rigid_body() -> RigidBody;
}
