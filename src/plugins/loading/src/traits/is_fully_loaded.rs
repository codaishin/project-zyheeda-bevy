mod asset_server;

use bevy::prelude::*;

pub trait IsFullyLoaded {
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset;
}

impl<TResource> IsFullyLoaded for Res<'_, TResource>
where
	TResource: Resource + IsFullyLoaded,
{
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset,
	{
		TResource::is_fully_loaded(self, id)
	}
}

#[cfg(test)]
impl<TInner> IsFullyLoaded for In<TInner>
where
	TInner: IsFullyLoaded,
{
	fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
	where
		TAsset: Asset,
	{
		self.0.is_fully_loaded(id)
	}
}
