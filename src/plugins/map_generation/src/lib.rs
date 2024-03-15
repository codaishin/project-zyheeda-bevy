mod components;
mod map;
mod map_loader;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Startup, Update},
	asset::{AssetApp, AssetServer},
	ecs::system::IntoSystem,
};
use components::{Corner, Wall};
use map::{Map, MapCell};
use map_loader::TextLoader;
use systems::{
	add_colliders::add_colliders,
	begin_level_load::begin_level_load,
	get_cell_transforms::get_cell_transforms,
	spawn_as_scene::spawn_as_scene,
};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Map<MapCell>>()
			.register_asset_loader(TextLoader::<Map<MapCell>>::default())
			.add_systems(Startup, (begin_level_load::<AssetServer, MapCell>,))
			.add_systems(
				Update,
				get_cell_transforms::<MapCell>.pipe(spawn_as_scene::<MapCell, AssetServer>),
			)
			.add_systems(Update, (add_colliders::<Wall>, add_colliders::<Corner>));
	}
}
