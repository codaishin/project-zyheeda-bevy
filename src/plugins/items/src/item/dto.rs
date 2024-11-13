use super::Item;
use bevy::reflect::TypePath;
use common::traits::load_asset::LoadAsset;
use loading::traits::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemDto<TContentDto>
where
	TContentDto: AssetFileExtensions,
{
	name: String,
	content: TContentDto,
}

impl<TContentDto, TContent> LoadFrom<ItemDto<TContentDto>> for Item<TContent>
where
	TContent: LoadFrom<TContentDto> + TypePath + Sync + Send + 'static,
	TContentDto: AssetFileExtensions + TypePath,
{
	fn load_from<TLoadAsset: LoadAsset>(
		from: ItemDto<TContentDto>,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			name: from.name,
			content: TContent::load_from(from.content, asset_server),
		}
	}
}

impl<TContentDto> AssetFileExtensions for ItemDto<TContentDto>
where
	TContentDto: AssetFileExtensions,
{
	fn asset_file_extensions() -> &'static [&'static str] {
		TContentDto::asset_file_extensions()
	}
}
