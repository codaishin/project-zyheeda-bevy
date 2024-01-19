pub mod dummy;
pub mod projectile;

use bevy::{asset::Asset, render::mesh::Mesh};

pub trait Model<TMaterial: Asset> {
	fn material() -> TMaterial;
	fn mesh() -> Mesh;
}
