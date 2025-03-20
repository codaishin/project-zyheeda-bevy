use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::{components::insert_asset::InsertAsset, traits::handles_lights::HandlesLights};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct WallLight;

impl ExtraComponentsDefinition for WallLight {
	fn target_names() -> Vec<String> {
		vec!["WallLightData".to_owned()]
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands)
	where
		TLights: HandlesLights,
	{
		entity.insert((
			WallLight,
			InsertAsset::shared::<WallLight>(|| StandardMaterial {
				base_color: Color::from(TLights::DEFAULT_LIGHT),
				emissive: LinearRgba::from(TLights::DEFAULT_LIGHT),
				..default()
			}),
		));
	}
}
