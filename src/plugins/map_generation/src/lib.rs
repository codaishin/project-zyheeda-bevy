mod components;
mod map;
mod map_loader;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Startup, Update},
	asset::{AssetApp, AssetServer},
};
use components::{Corner, Wall};
use map::{Cell, Map};
use map_loader::TextLoader;
use systems::{
	add_colliders::add_colliders,
	begin_level_load::begin_level_load,
	finish_level_load::finish_level_load,
};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Map>()
			.register_asset_loader(TextLoader::<Map>::default())
			.add_systems(Startup, begin_level_load::<AssetServer>)
			.add_systems(Update, finish_level_load::<AssetServer, Map, Cell>)
			.add_systems(Update, (add_colliders::<Wall>, add_colliders::<Corner>));
	}
}
