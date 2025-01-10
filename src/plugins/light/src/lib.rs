pub mod components;

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
	thread_safe::ThreadSafe,
};
use components::{
	responsive_light::ResponsiveLight,
	responsive_light_trigger::ResponsiveLightTrigger,
};
use std::marker::PhantomData;

pub struct LightPlugin<TPrefabs>(PhantomData<TPrefabs>);

impl<TPrefabs> LightPlugin<TPrefabs>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
{
	pub fn depends_on(_: &TPrefabs) -> Self {
		Self(PhantomData)
	}
}

impl<TPrefabs> Plugin for LightPlugin<TPrefabs>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
{
	fn build(&self, app: &mut App) {
		TPrefabs::register_prefab::<ResponsiveLight>(app);

		app.insert_resource(AmbientLight::NONE).add_systems(
			Update,
			(
				ResponsiveLight::detect_change::<CollidingEntities>,
				ResponsiveLight::apply_change::<Virtual>,
			)
				.chain(),
		);
	}
}

impl<TPrefabs> HandlesLights for LightPlugin<TPrefabs> {
	type TResponsiveLightBundle = ResponsiveLight;
	type TResponsiveLightTrigger = ResponsiveLightTrigger;

	fn responsive_light_bundle(responsive_light: Responsive) -> Self::TResponsiveLightBundle {
		ResponsiveLight::from(responsive_light)
	}

	fn responsive_light_trigger() -> Self::TResponsiveLightTrigger {
		ResponsiveLightTrigger
	}
}
