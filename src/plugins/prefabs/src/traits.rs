pub mod app;
pub(crate) mod dummy;
pub(crate) mod projectile;

use bevy::{
	asset::Handle,
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
pub enum AssetKey {
	Projectile(ProjectileType),
	Dummy,
	VoidSphere(VoidPart),
	Beam,
}

pub trait Instantiate {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error>;
}

pub trait RegisterPrefab {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self;
}

pub fn sphere(radius: f32, _: fn() -> &'static str) -> Result<Mesh, Error> {
	Ok(Mesh::from(Sphere { radius }))
}
