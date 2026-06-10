mod components;
mod system_params;
mod systems;

use crate::{
	components::{light::Light, torch_light::TorchLight},
	system_params::lights::{Lights, LightsMut},
};
use bevy::{camera::visibility::VisibilitySystems, prelude::*};
use common::traits::{handles_light::HandlesLight, prefab::AddPrefabObserver};

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(GlobalAmbientLight::NONE)
			.add_prefab_observer::<TorchLight, ()>()
			.add_systems(
				PostUpdate,
				Light::set_visibility.in_set(VisibilitySystems::CheckVisibility),
			);
	}
}

impl HandlesLight for LightPlugin {
	type TLights = Lights<'static, 'static>;
	type TLightsMut = LightsMut<'static, 'static>;
}
