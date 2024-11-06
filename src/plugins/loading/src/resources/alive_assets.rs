use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AliveAssets<TAsset: Asset>(HashSet<Handle<TAsset>>);

impl<TAsset: Asset> Default for AliveAssets<TAsset> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<TAsset: Asset> AliveAssets<TAsset> {
	#[cfg(test)]
	pub(crate) fn iter(&self) -> impl Iterator<Item = &Handle<TAsset>> {
		self.0.iter()
	}

	pub(crate) fn insert(&mut self, handle: Handle<TAsset>) {
		self.0.insert(handle);
	}
}
