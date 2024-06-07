use std::any::TypeId;

use super::{Instantiate, RegisterPrefab};
use crate::systems::instantiate::instantiate;
use bevy::{
	app::{App, PreUpdate},
	asset::{Assets, Handle},
	ecs::{component::Component, system::IntoSystem},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{resources::Shared, systems::log::log_many};

impl RegisterPrefab for App {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self {
		let instantiate_system = instantiate::<
			TPrefab,
			Assets<Mesh>,
			Assets<StandardMaterial>,
			Shared<TypeId, Handle<Mesh>>,
			Shared<TypeId, Handle<StandardMaterial>>,
		>;
		self.add_systems(PreUpdate, instantiate_system.pipe(log_many))
	}
}
