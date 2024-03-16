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
use components::{Corner, Light, Point, Wall};
use map::{LightCell, Map, MapCell};
use map_loader::TextLoader;
use prefabs::traits::RegisterPrefab;
use systems::{
	add_colliders::add_colliders,
	begin_level_load::begin_level_load,
	get_cell_transforms::get_cell_transforms,
	spawn_procedural::spawn_procedural,
	spawn_scene::spawn_scene,
};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.register_prefab::<Light<Point>>()
			.init_asset::<Map<MapCell>>()
			.init_asset::<Map<LightCell>>()
			.register_asset_loader(TextLoader::<Map<MapCell>>::default())
			.register_asset_loader(TextLoader::<Map<LightCell>>::default())
			.add_systems(
				Startup,
				(
					begin_level_load::<AssetServer, MapCell>,
					begin_level_load::<AssetServer, LightCell>,
				),
			)
			.add_systems(
				Update,
				(
					get_cell_transforms::<MapCell>.pipe(spawn_scene::<MapCell, AssetServer>),
					get_cell_transforms::<LightCell>.pipe(spawn_procedural),
				),
			)
			.add_systems(Update, (add_colliders::<Wall>, add_colliders::<Corner>));
	}
}
