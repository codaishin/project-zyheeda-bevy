pub mod components;

mod systems;
mod traits;

use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_rapier3d::geometry::CollidingEntities;
use common::{
	states::game_state::GameState,
	traits::{
		handles_lights::{HandlesLights, Responsive},
		prefab::AddPrefabObserver,
	},
};
use components::{
	responsive_light::ResponsiveLight,
	responsive_light_trigger::ResponsiveLightTrigger,
};
use systems::setup_light::setup_light;

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.add_prefab_observer::<ResponsiveLight, ()>()
			.add_systems(
				OnEnter(GameState::Loading),
				setup_light(Self::DEFAULT_LIGHT),
			)
			.add_systems(
				Update,
				(
					ResponsiveLight::insert_light,
					ResponsiveLight::detect_change::<CollidingEntities>,
					ResponsiveLight::apply_change::<Virtual, PointLight>,
					ResponsiveLight::apply_change::<Virtual, SpotLight>,
					ResponsiveLight::apply_change::<Virtual, DirectionalLight>,
				)
					.chain(),
			);
	}
}

impl HandlesLights for LightPlugin {
	type TResponsiveLightBundle = ResponsiveLight;
	type TResponsiveLightTrigger = ResponsiveLightTrigger;

	const DEFAULT_LIGHT: Srgba = WHITE;

	fn responsive_light_bundle<TDriver>(responsive_data: Responsive) -> Self::TResponsiveLightBundle
	where
		TDriver: 'static,
	{
		ResponsiveLight::for_driver::<TDriver>(responsive_data)
	}

	fn responsive_light_trigger() -> Self::TResponsiveLightTrigger {
		ResponsiveLightTrigger
	}
}
