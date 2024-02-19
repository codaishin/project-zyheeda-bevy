mod systems;
pub mod traits;

use bevy::{
	app::{Plugin, Update},
	asset::Handle,
	ecs::system::IntoSystem,
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{
	components::{Plasma, Projectile, VoidSphere},
	resources::Shared,
	systems::log::log_many,
};
use systems::instantiate::instantiate;
use traits::AssetKey;

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.init_resource::<Shared<AssetKey, Handle<Mesh>>>()
			.init_resource::<Shared<AssetKey, Handle<StandardMaterial>>>()
			.add_systems(Update, instantiate::<Projectile<Plasma>>.pipe(log_many))
			.add_systems(Update, instantiate::<VoidSphere>.pipe(log_many));
	}
}
