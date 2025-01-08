use super::Item;
use crate::{components::model_render::ModelRender, item::item_type::SkillItemType};
use bevy::reflect::TypePath;
use common::{
	components::essence::Essence,
	traits::{
		handles_custom_assets::{AssetFileExtensions, LoadFrom},
		load_asset::LoadAsset,
	},
};
use serde::{Deserialize, Serialize};

type SkillPath = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub(crate) struct ItemDto {
	name: String,
	model: ModelRender,
	essence: Essence,
	skill: Option<SkillPath>,
	item_type: SkillItemType,
}

impl LoadFrom<ItemDto> for Item {
	fn load_from<TLoadAsset: LoadAsset>(from: ItemDto, asset_server: &mut TLoadAsset) -> Self {
		Self {
			name: from.name,
			model: from.model,
			essence: from.essence,
			skill: from.skill.map(|path| asset_server.load_asset(path)),
			item_type: from.item_type,
		}
	}
}

impl AssetFileExtensions for ItemDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&[".item"]
	}
}
