use super::material::MaterialDto;
use crate::{
	components::renderer::{EssenceRender, ModelRender, Renderer},
	item::{item_type::SkillItemType, SkillItemContent},
};
use bevy::reflect::TypePath;
use common::traits::load_asset::LoadAsset;
use loading::traits::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom};
use serde::{Deserialize, Serialize};

type SkillPath = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub(crate) struct SkillItemContentDto {
	model: ModelRender,
	material: MaterialDto,
	skill: Option<SkillPath>,
	item_type: SkillItemType,
}

impl LoadFrom<SkillItemContentDto> for SkillItemContent {
	fn load_from<TLoadAsset: LoadAsset>(
		from: SkillItemContentDto,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			render: Renderer {
				model: from.model,
				essence: EssenceRender::from(from.material),
			},
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
