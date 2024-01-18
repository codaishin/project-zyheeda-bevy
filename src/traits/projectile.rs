#[allow(clippy::module_inception)]
pub mod projectile;

use bevy::{math::Vec3, pbr::StandardMaterial, render::mesh::Mesh};

pub trait ProjectileModelData {
	fn material() -> StandardMaterial;
	fn mesh() -> Mesh;
}

pub trait ProjectileBehaviorData {
	fn direction(&self) -> Vec3;
	fn range(&self) -> f32;
}
