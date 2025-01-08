pub mod attributes;
pub mod blocker;
pub mod components;
pub mod dto;
pub mod effects;
pub mod errors;
pub mod labels;
pub mod resources;
pub mod states;
pub mod systems;
pub mod test_tools;
pub mod tools;
pub mod traits;

use bevy::prelude::*;
use components::flip::FlipHorizontally;
use resources::language_server::LanguageServer;
use systems::load_asset_model::load_asset_model;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<LanguageServer>()
			.add_systems(First, load_asset_model::<AssetServer>)
			.add_systems(Update, FlipHorizontally::system);
	}
}
