pub mod components;
pub(crate) mod systems;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::IntoSystemConfigs,
	pbr::AmbientLight,
	time::Virtual,
};
use bevy_rapier3d::geometry::CollidingEntities;
use common::traits::handles_lights::{HandlesLights, Responsive};
use components::responsive_light::ResponsiveLight;
use systems::{
	apply_responsive_light_change::apply_responsive_light_change,
	detect_responsive_light_change::detect_responsive_light_change,
	insert_responsive_light_collider::insert_responsive_light_collider,
};

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(AmbientLight::NONE)
			.add_systems(Update, insert_responsive_light_collider)
			.add_systems(
				Update,
				(
					detect_responsive_light_change::<CollidingEntities>,
					apply_responsive_light_change::<Virtual>,
				)
					.chain(),
			);
	}
}

impl HandlesLights for LightPlugin {
	type TResponsiveLightBundle = ResponsiveLight;

	fn responsive_light_bundle(responsive_light: Responsive) -> Self::TResponsiveLightBundle {
		ResponsiveLight::from(responsive_light)
	}
}
