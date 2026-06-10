mod components;
mod observers;
mod systems;

use crate::{
	components::{corridor_light::CorridorLight, light::Light, wall_light::WallLight},
	observers::get_insert_system::GetInsertObserver,
};
use bevy::{camera::visibility::VisibilitySystems, prelude::*};

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(GlobalAmbientLight::NONE)
			.add_observer(WallLight::get_insert_observer())
			.add_observer(CorridorLight::get_insert_observer())
			.add_systems(
				PostUpdate,
				Light::set_visibility.in_set(VisibilitySystems::CheckVisibility),
			);
	}
}
