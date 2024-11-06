use super::IsFullyLoaded;
use bevy::prelude::*;

impl IsFullyLoaded for AssetServer {
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset,
	{
		self.is_loaded_with_dependencies(id)
	}
}
