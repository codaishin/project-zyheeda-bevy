use super::{
	handles_custom_assets::{AssetFileExtensions, LoadFrom},
	handles_load_tracking::LoadGroup,
	load_asset::Path,
	thread_safe::ThreadSafe,
};
use bevy::prelude::*;
use serde::Deserialize;
use std::fmt::Debug;

pub trait HandlesAssetResourceLoading {
	fn register_custom_resource_loading<TResource, TDto, TLoadGroup>(app: &mut App, path: Path)
	where
		TResource: Resource + Asset + Clone + LoadFrom<TDto> + Debug,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + AssetFileExtensions,
		TLoadGroup: LoadGroup + ThreadSafe;
}
