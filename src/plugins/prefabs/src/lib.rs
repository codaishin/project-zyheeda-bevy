mod systems;
pub mod traits;

use std::any::TypeId;

use bevy::{
	app::{App, Plugin},
	asset::Handle,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::resources::Shared;

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>()
			.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>();
	}
}
