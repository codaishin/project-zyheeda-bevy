use crate::{components::Unlit, traits::ExtraComponentsDefinition};
use bevy::{color::palettes::css::BLACK, prelude::*};
use common::{components::insert_asset::InsertAsset, zyheeda_commands::ZyheedaEntityCommands};

#[derive(Component)]
pub(crate) struct WallBack;

impl ExtraComponentsDefinition for WallBack {
	fn target_names() -> Vec<String> {
		WALL_PARTS
			.iter()
			.map(|part| format!("Wall{part}BackData"))
			.collect()
	}

	fn insert_bundle<TLights>(entity: &mut ZyheedaEntityCommands) {
		entity.try_insert((
			Unlit,
			WallBack,
			InsertAsset::shared::<WallBack>(|| StandardMaterial {
				base_color: Color::from(BLACK),
				..default()
			}),
		));
	}
}

const WALL_PARTS: &[&str] = &[
	"",
	"Forward",
	"Left",
	"CornerOutside",
	"CornerOutsideDiagonal",
	"CornerInside",
];
