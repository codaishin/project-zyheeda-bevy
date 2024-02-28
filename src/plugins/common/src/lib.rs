pub mod bundles;
pub mod components;
pub mod errors;
pub mod resources;
pub mod systems;
pub mod test_tools;
pub mod tools;
pub mod traits;

use bevy::{
	app::{App, First, Plugin},
	ecs::schedule::IntoSystemConfigs,
	render::camera::Camera,
};
use bevy_rapier3d::plugin::RapierContext;
use systems::{set_cam_ray::set_cam_ray, set_mouse_hover::set_mouse_hover};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			First,
			(set_cam_ray::<Camera>, set_mouse_hover::<RapierContext>).chain(),
		);
	}
}
