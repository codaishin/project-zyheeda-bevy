pub mod app;
pub(crate) mod dummy;
pub(crate) mod projectile;

use bevy::{
	asset::{Asset, Handle},
	ecs::{component::Component, system::EntityCommands},
	math::primitives::Sphere,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::errors::Error;

pub trait AssetHandleFor<TAsset: Asset> {
	fn handle<Key: 'static>(&mut self, asset: &mut dyn FnMut() -> TAsset) -> Handle<TAsset>;
}

pub trait AssetHandles: AssetHandleFor<Mesh> + AssetHandleFor<StandardMaterial> {}

impl<TAssetHandles> AssetHandles for TAssetHandles where
	TAssetHandles: AssetHandleFor<Mesh> + AssetHandleFor<StandardMaterial>
{
}

pub trait Instantiate {
	fn instantiate(&self, on: &mut EntityCommands, assets: impl AssetHandles) -> Result<(), Error>;
}

pub trait RegisterPrefab {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self;
}

pub fn sphere(radius: f32) -> Mesh {
	Mesh::from(Sphere { radius })
}
