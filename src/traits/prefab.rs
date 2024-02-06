pub mod dummy;
pub mod projectile;
pub mod simple_model;

use crate::errors::Error;
use bevy::{
	asset::{Asset, Assets},
	ecs::{
		bundle::Bundle,
		entity::Entity,
		system::{EntityCommands, ResMut},
	},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};

pub trait CreatePrefab<TPrefab, TMaterial: Asset = StandardMaterial> {
	fn create_prefab(
		materials: ResMut<Assets<TMaterial>>,
		meshes: ResMut<Assets<Mesh>>,
	) -> Result<TPrefab, Error>;
}

pub trait SpawnPrefab<TRootReference: From<Entity> + Bundle> {
	fn spawn_prefab(&self, parent: &mut EntityCommands);
}
