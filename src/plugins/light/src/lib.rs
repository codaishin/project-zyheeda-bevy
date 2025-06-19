pub mod components;

mod systems;
mod traits;

use crate::components::global_light::GlobalLight;
use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_rapier3d::geometry::CollidingEntities;
use common::{
	states::game_state::GameState,
	traits::{
		handles_lights::{HandlesLights, Responsive},
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		register_required_components_mapped::RegisterRequiredComponentsMapped,
		thread_safe::ThreadSafe,
	},
};
use components::{
	responsive_light::ResponsiveLight,
	responsive_light_trigger::ResponsiveLightTrigger,
};
use std::marker::PhantomData;

pub struct LightPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSavegame> LightPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSavegame) -> Self {
		Self(PhantomData)
	}
}

impl<TSavegame> Plugin for LightPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		TSavegame::register_savable_component::<GlobalLight>(app);

		app.add_prefab_observer::<ResponsiveLight, ()>()
			.register_required_components::<GlobalLight, TSavegame::TSaveEntityMarker>()
			.register_required_components_mapped::<GlobalLight, DirectionalLight>(
				GlobalLight::light,
			)
			.add_systems(
				OnEnter(GameState::NewGame),
				GlobalLight::spawn(Self::DEFAULT_LIGHT),
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

impl<TSavegame> HandlesLights for LightPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
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
