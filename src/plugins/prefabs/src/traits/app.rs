use super::{Instantiate, RegisterPrefab};
use crate::systems::instantiate::instantiate;
use bevy::{
	app::{App, Update},
	ecs::{component::Component, system::IntoSystem},
};
use common::systems::log::log_many;

impl RegisterPrefab for App {
	fn register_prefab<TPrefab: Instantiate + Component>(&mut self) -> &mut Self {
		self.add_systems(Update, instantiate::<TPrefab>.pipe(log_many))
	}
}
