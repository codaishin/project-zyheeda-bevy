use super::Item;
use crate::components::model_render::ModelRender;
use bevy::reflect::TypePath;
use common::{
	components::essence::Essence,
	tools::item_type::ItemType,
	traits::{
		handles_custom_assets::{AssetFileExtensions, LoadFrom},
		handles_localization::Token,
		load_asset::LoadAsset,
	},
};
use serde::{Deserialize, Serialize};

type SkillPath = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub(crate) struct ItemDto {
	token: String,
	model: ModelRender,
	essence: Essence,
	skill: Option<SkillPath>,
	item_type: ItemType,
}

impl LoadFrom<ItemDto> for Item {
	fn load_from<TLoadAsset: LoadAsset>(from: ItemDto, asset_server: &mut TLoadAsset) -> Self {
		Self {
			token: Token::from(from.token),
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
