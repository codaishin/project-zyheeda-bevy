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

pub struct LightPlugin<TDependencies>(PhantomData<TDependencies>);

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

impl<TDependencies> HandlesLights for LightPlugin<TDependencies> {
	type TResponsiveLightBundle = ResponsiveLight;
	type TResponsiveLightTrigger = ResponsiveLightTrigger;

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
