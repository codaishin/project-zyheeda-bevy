pub mod dummy;
pub mod projectile;

use bevy::{asset::Asset, math::Vec3, render::mesh::Mesh};

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
