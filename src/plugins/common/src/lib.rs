pub mod bundles;
pub mod components;
pub mod errors;
pub mod labels;
pub mod resources;
pub mod states;
pub mod systems;
pub mod test_tools;
pub mod tools;
pub mod traits;

pub(crate) mod folder_asset_loader;

use bevy::{
	app::{App, First, Plugin, Update},
	asset::AssetServer,
	ecs::schedule::IntoSystemConfigs,
	render::camera::Camera,
	state::app::AppExtStates,
};
use bevy_rapier3d::plugin::RapierContext;
use components::{flip::FlipHorizontally, MainCamera};
use resources::language_server::LanguageServer;
use states::{GameRunning, MouseContext};
use systems::{
	load_asset_model::load_asset_model,
	set_cam_ray::set_cam_ray,
	set_mouse_hover::set_mouse_hover,
};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<LanguageServer>()
			.init_state::<GameRunning>()
			.init_state::<MouseContext>()
			.add_systems(
				First,
				(
					set_cam_ray::<Camera, MainCamera>,
					set_mouse_hover::<RapierContext>,
				)
					.chain(),
			)
			.add_systems(First, load_asset_model::<AssetServer>)
			.add_systems(Update, FlipHorizontally::system);
	}
}
