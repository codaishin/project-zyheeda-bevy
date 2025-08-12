use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::{
	components::insert_asset::InsertAsset,
	traits::handles_lights::HandlesLights,
	zyheeda_commands::ZyheedaEntityCommands,
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct WallLight;

impl ExtraComponentsDefinition for WallLight {
	fn target_names() -> Vec<String> {
		WALL_PARTS
			.iter()
			.map(|part| format!("Wall{part}LightData"))
			.collect()
	}

	fn insert_bundle<TLights>(entity: &mut ZyheedaEntityCommands)
	where
		TLights: HandlesLights,
	{
		entity.try_insert((
			WallLight,
			InsertAsset::shared::<WallLight>(|| StandardMaterial {
				base_color: Color::from(TLights::DEFAULT_LIGHT),
				emissive: LinearRgba::from(TLights::DEFAULT_LIGHT),
				..default()
			}),
		));
	}
}

const WALL_PARTS: &[&str] = &[
	"",
	"Forward",
	"Left",
	"Corner",
	"CornerOutside",
	"CornerOutsideDiagonal",
	"CornerInside",
];
