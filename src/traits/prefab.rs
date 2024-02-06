pub mod dummy;
pub mod flat_model;
pub mod projectile;

use crate::errors::Error;
use bevy::{
	asset::{Asset, Assets},
	ecs::system::{EntityCommands, ResMut},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};

pub trait CreatePrefab<TPrefab, TMaterial: Asset = StandardMaterial> {
	fn create_prefab(
		materials: ResMut<Assets<TMaterial>>,
		meshes: ResMut<Assets<Mesh>>,
	) -> Result<TPrefab, Error>;
}

pub trait SpawnPrefab {
	fn spawn_prefab(&self, parent: &mut EntityCommands);
}
