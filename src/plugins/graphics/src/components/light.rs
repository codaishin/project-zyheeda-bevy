use crate::components::camera_labels::WorldLight;
use bevy::{camera::visibility::RenderLayers, ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(RenderLayers::from(WorldLight))]
pub(crate) struct Light(LightType);

impl Light {
	fn prefab(
		mut entity: ZyheedaEntityCommands,
		transform: GlobalTransform,
		light_type: LightType,
	) {
		match light_type {
			LightType::Roof => {
				let intensity = 1_000_000.0;
				let color = Color::WHITE;

				entity.try_insert((
					#[cfg(debug_assertions)]
					Name::from("RoofLight"),
					Light(LightType::Roof),
					Transform::from(transform).looking_to(Dir3::NEG_Y, Dir3::NEG_Z),
					SpotLight {
						color,
						intensity,
						..default()
					},
				));
			}
		}
	}

	pub(crate) fn configure_prefab<TMapGeneration>(mut param: StaticSystemParam<TMapGeneration>)
	where
		TMapGeneration: for<'c> GetContextMut<LightPrefabs, TContext<'c>: SetPrefab<LightType>>,
	{
		TMapGeneration::get_context_mut(&mut param, MapPrefabs::KEY).set_prefab(Self::prefab);
	}
}
