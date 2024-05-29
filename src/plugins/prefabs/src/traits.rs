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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectileType {
	Plasma,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoidPart {
	Core,
	Ring,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightStatus {
	On,
	Off,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightType {
	Floating,
	Wall(LightStatus),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKey {
	Projectile(ProjectileType),
	Dummy,
	VoidSphere(VoidPart),
	Beam,
	Light(LightType),
	ForceShield,
	GravityWell,
}

pub trait AssetHandleFor<TAsset: Asset> {
	fn handle<Key: 'static>(&mut self, asset: TAsset) -> Handle<TAsset>;
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
