pub mod app;
pub(crate) mod dummy;

use bevy::{
	ecs::{component::Component, system::EntityCommands},
	math::primitives::Sphere,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{errors::Error, traits::cache::GetOrCreateAsset};
use std::any::TypeId;

pub trait GetOrCreateAssets:
	GetOrCreateAsset<TypeId, Mesh> + GetOrCreateAsset<TypeId, StandardMaterial>
{
}

impl<TAssetHandles> GetOrCreateAssets for TAssetHandles where
	TAssetHandles: GetOrCreateAsset<TypeId, Mesh> + GetOrCreateAsset<TypeId, StandardMaterial>
{
}

pub trait Instantiate {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		assets: impl GetOrCreateAssets,
	) -> Result<(), Error>;
}

pub trait RegisterPrefab {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self;
}

pub fn sphere(radius: f32) -> Mesh {
	Mesh::from(Sphere { radius })
}
