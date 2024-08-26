use super::{Instantiate, RegisterPrefab};
use crate::systems::instantiate::instantiate;
use bevy::{
	app::App,
	asset::{Assets, Handle},
	ecs::{component::Component, system::IntoSystem},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{
	labels::Labels,
	resources::Shared,
	systems::log::log_many,
	tools::Factory,
	traits::cache::get_or_create_asset::CreateAssetCache,
};
use std::any::TypeId;

impl RegisterPrefab for App {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self {
		let instantiate_system = instantiate::<
			TPrefab,
			Assets<Mesh>,
			Assets<StandardMaterial>,
			Shared<TypeId, Handle<Mesh>>,
			Shared<TypeId, Handle<StandardMaterial>>,
			Factory<CreateAssetCache>,
		>;
		self.add_systems(
			Labels::INSTANTIATION.label(),
			instantiate_system.pipe(log_many),
		)
	}
}
