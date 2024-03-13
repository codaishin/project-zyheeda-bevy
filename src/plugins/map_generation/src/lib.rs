mod components;
mod map_loader;
mod parsers;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Startup, Update},
	asset::{AssetApp, AssetServer, Assets},
	ecs::system::{Commands, Res},
};
use components::{Corner, LoadLevelCommand, Wall};
use map_loader::{Map, MapLoader};
use parsers::ParseStringToCells;
use systems::{add_colliders::add_colliders, begin_level_load::begin_level_load};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Map>()
			.register_asset_loader(MapLoader::<ParseStringToCells>::default())
			.add_systems(Startup, begin_level_load::<AssetServer>)
			.add_systems(Update, (add_colliders::<Wall>, add_colliders::<Corner>))
			.add_systems(
				Update,
				|mut commands: Commands,
				 maps: Res<Assets<Map>>,
				 load_level: Option<Res<LoadLevelCommand>>| {
					let Some(map) = load_level.and_then(|load| maps.get(&load.0)) else {
						return;
					};
					println!("{:?}", map.0);
					commands.remove_resource::<LoadLevelCommand>();
				},
			);
	}
}
