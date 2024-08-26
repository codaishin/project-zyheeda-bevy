mod systems;

pub mod components;
pub mod traits;

use std::any::TypeId;

use bevy::{
	app::{App, Plugin},
	asset::Handle,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{labels::Labels, resources::Shared};
use systems::instantiate_children::instantiate_children;

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>()
			.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>()
			.add_systems(Labels::INSTANTIATION.label(), instantiate_children);
	}
}
