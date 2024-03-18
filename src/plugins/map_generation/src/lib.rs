mod components;
mod map;
mod map_loader;
mod systems;
mod traits;
mod types;

use bevy::{
	app::{App, Plugin, Startup, Update},
	asset::AssetServer,
	ecs::system::IntoSystem,
};
use bevy_rapier3d::geometry::Collider;
use common::components::NoTarget;
use components::{Corner, Floating, Light, Wall};
use map::{LightCell, MapCell};
use prefabs::traits::RegisterPrefab;
use systems::{
	add_component::add_component,
	get_cell_transforms::get_cell_transforms,
	spawn_procedural::spawn_procedural,
	spawn_scene::spawn_scene,
};
use traits::RegisterMapCell;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.register_prefab::<Light<Floating>>()
			.register_prefab::<Light<Wall>>()
			.register_map_cell::<MapCell>(Startup)
			.register_map_cell::<LightCell>(Startup)
			.add_systems(
				Update,
				(
					get_cell_transforms::<MapCell>.pipe(spawn_scene::<MapCell, AssetServer>),
					get_cell_transforms::<LightCell>.pipe(spawn_procedural),
				),
			)
			.add_systems(
				Update,
				(
					add_component::<Wall, (Collider, NoTarget)>,
					add_component::<Corner, (Collider, NoTarget)>,
					add_component::<Light<Wall>, Light<Wall>>,
				),
			);
	}
}
