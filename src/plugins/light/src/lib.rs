mod components;
mod systems;

use crate::components::global_light::GlobalLight;
use bevy::{color::palettes::css::WHITE, prelude::*};
use common::traits::{
	handles_lights::HandlesLights,
	handles_saving::HandlesSaving,
	register_derived_component::RegisterDerivedComponent,
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
		TSavegame::register_savable_component::<GlobalLight>(app);

		app.register_required_components::<GlobalLight, TSavegame::TSaveEntityMarker>()
			.register_derived_component::<GlobalLight, DirectionalLight>()
			.add_systems(Startup, GlobalLight::spawn(Self::DEFAULT_LIGHT));
	}
}

impl<TDependencies> HandlesLights for LightPlugin<TDependencies> {
	const DEFAULT_LIGHT: Srgba = WHITE;
}
