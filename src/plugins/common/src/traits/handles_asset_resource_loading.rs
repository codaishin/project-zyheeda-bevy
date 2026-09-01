use super::{
	handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
	handles_load_tracking::LoadGroup,
	thread_safe::ThreadSafe,
};
use crate::tools::path::Path;
use bevy::prelude::*;
use serde::Deserialize;
use std::{error::Error, fmt::Debug};

pub trait HandlesAssetResourceLoading {
	type TLoadAssetState: Default;

	fn register_custom_resource_loading<TResource, TDto>(
		app: &mut App,
		load_group: impl LoadGroup<Self::TLoadAssetState>,
		path: Path,
	) where
		TResource: Resource
			+ Asset
			+ Clone
			+ TryLoadFrom<TDto, TInstantiationError: Error + TypePath + ThreadSafe>
			+ Debug,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + TypePath + AssetFileExtensions;
}
