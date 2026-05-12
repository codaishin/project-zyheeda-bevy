mod components;
mod observers;
mod systems;

use crate::{
	components::{corridor_light::CorridorLight, light::Light, wall_light::WallLight},
	observers::get_insert_system::GetInsertObserver,
};
use bevy::{camera::visibility::VisibilitySystems, color::palettes::css::WHITE, prelude::*};
use common::traits::{
	handles_lights::HandlesLights,
	handles_saving::HandlesSaving,
	thread_safe::ThreadSafe,
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
		app.insert_resource(GlobalAmbientLight::NONE)
			.add_observer(WallLight::get_insert_observer())
			.add_observer(CorridorLight::get_insert_observer())
			.add_systems(
				PostUpdate,
				Light::set_visibility.in_set(VisibilitySystems::CheckVisibility),
			);
	}
}

impl<TDependencies> HandlesLights for LightPlugin<TDependencies> {
	const DEFAULT_LIGHT: Srgba = WHITE;
}
