pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::IntoSystemConfigs,
	render::camera::Camera,
};
use common::{components::Health, traits::ownership_relation::OwnershipRelation};
use components::Bar;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin;

impl Plugin for BarsPlugin {
	fn build(&self, app: &mut App) {
		app.manage_ownership::<Bar>(Update);
		app.add_systems(
			Update,
			(bar::<Health, Camera>, render_bar::<Health>).chain(),
		);
	}
}
