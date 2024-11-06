mod asset_server;

use bevy::prelude::*;
use std::ops::Deref;

pub trait IsFullyLoaded {
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset;
}

impl<'w, T> IsFullyLoaded for Res<'w, T>
where
	T: Resource + IsFullyLoaded,
{
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset,
	{
		self.deref().is_fully_loaded(id)
	}
}

#[cfg(test)]
impl<T> IsFullyLoaded for In<T>
where
	T: IsFullyLoaded,
{
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset,
	{
		self.0.is_fully_loaded(id)
	}
}
