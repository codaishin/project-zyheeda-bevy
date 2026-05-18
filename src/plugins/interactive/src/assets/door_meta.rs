use bevy::prelude::*;
use common::{
	tools::path::Path,
	traits::{
		handles_animations::AnimationName,
		handles_custom_assets::{AssetFileExtensions, AssetFolderPath},
	},
};
use macros::asset_path;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DoorMeta {
	animations: DoorAnimations,
}

impl AssetFolderPath for DoorMeta {
	fn asset_folder_path() -> Path {
		Path::from(asset_path!("maps"))
	}
}

impl AssetFileExtensions for DoorMeta {
	fn asset_file_extensions() -> &'static [&'static str] {
		const { &["door"] }
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DoorAnimations {
	open: AnimationName,
	close: AnimationName,
}
