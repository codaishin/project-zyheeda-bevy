use super::Item;
use crate::components::model_render::ModelRender;
use bevy::reflect::TypePath;
use common::{components::essence::Essence, prelude::*};
use macros::serde_model;

type SkillPath = String;

#[serde_model]
#[derive(Debug, PartialEq, TypePath)]
pub(crate) struct ItemDto {
	token: String,
	model: ModelRender,
	essence: Essence,
	skill: Option<SkillPath>,
	item_type: ItemType,
}

impl TryLoadFrom<ItemDto> for Item {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset: LoadAsset>(
		from: ItemDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			token: Token::from(from.token),
			model: from.model,
			essence: from.essence,
			skill: from.skill.map(|path| asset_server.load_asset(path)),
			item_type: from.item_type,
		})
	}
}

impl AssetFileExtensions for ItemDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["item"]
	}
}
