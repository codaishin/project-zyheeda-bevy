mod systems;

pub mod components;
pub mod traits;

use std::any::TypeId;

use bevy::{
	app::{App, Plugin, PreUpdate},
	asset::Handle,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::resources::Shared;
use systems::instantiate_children::instantiate_children;

pub struct PrefabsPlugin;

impl PrefabsPlugin {
	const INSTANTIATION_FRAME: PreUpdate = PreUpdate;
}

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>()
			.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>()
			.add_systems(Self::INSTANTIATION_FRAME, instantiate_children);
	}
}
