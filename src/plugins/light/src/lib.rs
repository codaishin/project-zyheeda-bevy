mod components;
mod observers;
mod system_params;
mod systems;

use crate::{
	components::{
		corridor_light::CorridorLight,
		light::Light,
		torch_light::TorchLight,
		wall_light::WallLight,
	},
	observers::get_insert_system::GetInsertObserver,
	system_params::lights::{Lights, LightsMut},
};
use bevy::{camera::visibility::VisibilitySystems, prelude::*};
use common::traits::{handles_light::HandlesLight, prefab::AddPrefabObserver};

pub struct LightPlugin;

impl Plugin for LightPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(GlobalAmbientLight::NONE)
			.add_prefab_observer::<TorchLight, ()>()
			.add_observer(WallLight::get_insert_observer())
			.add_observer(CorridorLight::get_insert_observer())
			.add_systems(
				PostUpdate,
				Light::set_visibility.in_set(VisibilitySystems::CheckVisibility),
			);
	}
}

impl HandlesLight for LightPlugin {
	type Lights = Lights<'static, 'static>;
	type LightsMut = LightsMut<'static, 'static>;
}
