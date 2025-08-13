use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::{
	components::insert_asset::InsertAsset,
	traits::handles_lights::HandlesLights,
	zyheeda_commands::ZyheedaEntityCommands,
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct FloorLight;

impl ExtraComponentsDefinition for FloorLight {
	fn target_names() -> Vec<String> {
		vec!["FloorLightData".to_owned()]
	}

	fn insert_bundle<TLights>(entity: &mut ZyheedaEntityCommands)
	where
		TLights: HandlesLights,
	{
		entity.try_insert((
			FloorLight,
			InsertAsset::shared::<FloorLight>(|| StandardMaterial {
				base_color: Color::from(TLights::DEFAULT_LIGHT),
				emissive: LinearRgba::from(TLights::DEFAULT_LIGHT),
				..default()
			}),
		));
	}
}
