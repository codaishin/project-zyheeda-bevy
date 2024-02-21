mod systems;
pub mod traits;

use bevy::{
	app::{App, Plugin},
	asset::Handle,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::resources::Shared;
use traits::AssetKey;

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<AssetKey, Handle<Mesh>>>()
			.init_resource::<Shared<AssetKey, Handle<StandardMaterial>>>();
	}
}
