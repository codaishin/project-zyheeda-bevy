mod components;
mod map;
mod map_loader;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Startup, Update},
	asset::AssetServer,
	ecs::system::IntoSystem,
};
use components::{Corner, Floating, Light, Wall};
use map::{LightCell, MapCell};
use prefabs::traits::RegisterPrefab;
use systems::{
	add_colliders::add_colliders,
	get_cell_transforms::get_cell_transforms,
	spawn_procedural::spawn_procedural,
	spawn_scene::spawn_scene,
};
use traits::RegisterMapCell;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.register_prefab::<Light<Floating>>()
			.register_map_cell::<MapCell>(Startup)
			.register_map_cell::<LightCell>(Startup)
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
