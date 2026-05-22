use bevy::{platform::collections::HashMap, prelude::*};
use common::{
	systems::register_animations::{AnimationConfig, AnimationKeyAndNames, AnimationMaskAndBones},
	tools::path::Path,
	traits::{
		handles_animations::{
			AffectedAnimationBones,
			Animation,
			AnimationKey,
			AnimationMaskBits,
			AnimationNames,
		},
		handles_custom_assets::{AssetFileExtensions, AssetFolderPath},
		handles_physics::physical_bodies::ShapeParameters,
	},
};
use macros::asset_path;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DoorMeta {
	pub(crate) animations: DoorAnimations,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
	pub(crate) interactive_detection_shape: ShapeParameters,
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

impl AnimationConfig for DoorMeta {
	fn animations(&self) -> impl ExactSizeIterator<Item = AnimationKeyAndNames> {
		[
			(AnimationKey::Open, self.animations.open.clone()),
			(AnimationKey::Close, self.animations.close.clone()),
		]
		.into_iter()
	}

	fn masks(&self) -> impl ExactSizeIterator<Item = AnimationMaskAndBones> {
		self.animation_mask_groups
			.iter()
			.map(|(mask, bones)| (*mask, bones.clone()))
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DoorAnimations {
	open: Animation<AnimationNames>,
	close: Animation<AnimationNames>,
}
