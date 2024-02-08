pub mod complex_collidable;
pub mod dummy;
pub mod projectile;
pub mod simple_collidable;
pub mod void_sphere;

use crate::errors::{Error, Level};
use bevy::{
	asset::{Asset, Assets, Handle},
	ecs::system::{EntityCommands, ResMut},
	pbr::StandardMaterial,
	render::mesh::{shape::Icosphere, Mesh},
};

pub trait CreatePrefab<TPrefab, TMaterial: Asset = StandardMaterial> {
	fn create_prefab(
		materials: ResMut<Assets<TMaterial>>,
		meshes: ResMut<Assets<Mesh>>,
	) -> Result<TPrefab, Error>;
}

pub trait SpawnPrefab<TFor> {
	fn spawn_prefab(&self, parent: &mut EntityCommands);
}

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
}

pub trait Instantiate {
	fn instantiate(
		on: &mut EntityCommands,
		get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error>;
}

macro_rules! projectile_error {
	($t:expr, $e:expr) => {
		format!("{}: {}", $t, $e)
	};
}

fn sphere(radius: f32, error_msg: fn() -> &'static str) -> Result<Mesh, Error> {
	Mesh::try_from(Icosphere {
		radius,
		subdivisions: 5,
	})
	.map_err(|err| Error {
		lvl: Level::Error,
		msg: projectile_error!(error_msg(), err),
	})
}
