mod components;
mod map;
mod map_loader;
mod systems;
mod traits;

use bevy::prelude::*;
use common::states::game_state::GameState;
use components::{Floating, Light, Wall, WallBack};
use map::{LightCell, MapCell};
use prefabs::traits::RegisterPrefab;
use systems::{
	apply_extra_components::apply_extra_components,
	get_cell_transforms::get_cell_transforms,
	spawn_procedural::spawn_procedural,
	spawn_scene::spawn_scene,
	unlit_material::unlit_material,
};
use traits::RegisterMapCell;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		let new_game = GameState::NewGame;

		app.register_prefab::<Light<Floating>>()
			.register_prefab::<Light<Wall>>()
			.register_map_cell::<MapCell>(OnEnter(new_game))
			.register_map_cell::<LightCell>(OnEnter(new_game))
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
					apply_extra_components::<Wall>,
					apply_extra_components::<WallBack>,
					apply_extra_components::<Light<Wall>>,
				),
			)
			.add_systems(Update, unlit_material);
	}
}
