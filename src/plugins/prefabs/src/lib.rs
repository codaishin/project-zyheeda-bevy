mod components;
mod systems;

use bevy::prelude::*;
use common::{
	labels::Labels,
	resources::Shared,
	systems::log::log_many,
	tools::Factory,
	traits::{
		cache::get_or_create_asset::CreateAssetCache,
		prefab::{Instantiate, RegisterPrefab},
	},
};
use std::any::TypeId;
use systems::{instantiate::instantiate, instantiate_children::instantiate_children};

pub struct PrefabsPlugin;

impl RegisterPrefab for PrefabsPlugin {
	fn register_prefab<TPrefab: Instantiate + Component>(app: &mut App) {
		let instantiate_system = instantiate::<
			TPrefab,
			Assets<Mesh>,
			Assets<StandardMaterial>,
			Shared<TypeId, Handle<Mesh>>,
			Shared<TypeId, Handle<StandardMaterial>>,
			Factory<CreateAssetCache>,
		>;
		app.add_systems(
			Labels::INSTANTIATION.label(),
			instantiate_system.pipe(log_many),
		);
	}
}

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>()
			.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>()
			.add_systems(Labels::INSTANTIATION.label(), instantiate_children);
	}
}
