use super::{
	handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
	handles_load_tracking::LoadGroup,
	load_asset::Path,
	thread_safe::ThreadSafe,
};
use bevy::prelude::*;
use serde::Deserialize;
use std::{error::Error, fmt::Debug};

pub trait HandlesAssetResourceLoading {
	fn register_custom_resource_loading<TResource, TDto, TLoadGroup>(app: &mut App, path: Path)
	where
		TResource: Resource + Asset + Clone + TryLoadFrom<TDto> + Debug,
		TResource::TInstantiationError: Error + TypePath + ThreadSafe,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + AssetFileExtensions,
		TLoadGroup: LoadGroup + ThreadSafe;
}
