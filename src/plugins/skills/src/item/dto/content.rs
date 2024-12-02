use crate::{
	components::model_render::ModelRender,
	item::{item_type::SkillItemType, SkillItemContent},
};
use bevy::reflect::TypePath;
use common::{components::essence::Essence, traits::load_asset::LoadAsset};
use loading::traits::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom};
use serde::{Deserialize, Serialize};

type SkillPath = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub(crate) struct SkillItemContentDto {
	model: ModelRender,
	essence: Essence,
	skill: Option<SkillPath>,
	item_type: SkillItemType,
}

impl LoadFrom<SkillItemContentDto> for SkillItemContent {
	fn load_from<TLoadAsset: LoadAsset>(
		from: SkillItemContentDto,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			model: from.model,
			essence: from.essence,
			skill: from.skill.map(|path| asset_server.load_asset(path)),
			item_type: from.item_type,
		}
	}
}

impl AssetFileExtensions for SkillItemContentDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&[".item"]
	}
}