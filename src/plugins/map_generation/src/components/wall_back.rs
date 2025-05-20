use crate::{components::Unlit, traits::ExtraComponentsDefinition};
use bevy::{color::palettes::css::BLACK, ecs::system::EntityCommands, prelude::*};
use common::components::insert_asset::InsertAsset;

#[derive(Component)]
pub(crate) struct WallBack;

impl ExtraComponentsDefinition for WallBack {
	fn target_names() -> Vec<String> {
		WALL_PARTS
			.iter()
			.map(|part| format!("Wall{part}BackData"))
			.collect()
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands) {
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
