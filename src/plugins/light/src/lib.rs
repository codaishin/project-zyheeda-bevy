pub mod components;
pub(crate) mod systems;

use std::marker::PhantomData;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::IntoSystemConfigs,
	pbr::AmbientLight,
	time::Virtual,
};
use bevy_rapier3d::geometry::CollidingEntities;
use common::traits::{
	handles_lights::{HandlesLights, Responsive},
	prefab::RegisterPrefab,
};
use components::responsive_light::ResponsiveLight;
use systems::{
	apply_responsive_light_change::apply_responsive_light_change,
	detect_responsive_light_change::detect_responsive_light_change,
};

pub struct LightPlugin<TPrefabs>(PhantomData<TPrefabs>);

impl<TPrefabs> LightPlugin<TPrefabs>
where
	TPrefabs: Plugin + RegisterPrefab,
{
	pub fn depends_on(_: &TPrefabs) -> Self {
		Self(PhantomData)
	}
}

impl<TPrefabs> Plugin for LightPlugin<TPrefabs>
where
	TPrefabs: Plugin + RegisterPrefab,
{
	fn build(&self, app: &mut App) {
		TPrefabs::register_prefab::<ResponsiveLight>(app);

		app.insert_resource(AmbientLight::NONE).add_systems(
			Update,
			(
				detect_responsive_light_change::<CollidingEntities>,
				apply_responsive_light_change::<Virtual>,
			)
				.chain(),
		);
	}
}

impl<TPrefabs> HandlesLights for LightPlugin<TPrefabs> {
	type TResponsiveLightBundle = ResponsiveLight;

	fn responsive_light_bundle(responsive_light: Responsive) -> Self::TResponsiveLightBundle {
		ResponsiveLight::from(responsive_light)
	}
}
