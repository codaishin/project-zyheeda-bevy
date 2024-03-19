pub mod components;
pub(crate) mod systems;

use bevy::{
	app::{App, Plugin, Update},
	pbr::AmbientLight,
};
use bevy_rapier3d::geometry::CollidingEntities;
use systems::{
	insert_responsive_light_collider::insert_responsive_light_collider,
	responsive_light::responsive_light,
};

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(AmbientLight::NONE)
			.add_systems(Update, insert_responsive_light_collider)
			.add_systems(Update, responsive_light::<CollidingEntities>);
	}
}
