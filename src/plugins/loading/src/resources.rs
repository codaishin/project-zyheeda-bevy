pub mod track;

use bevy::{asset::LoadedFolder, prelude::*};
use std::{collections::HashSet, marker::PhantomData};

#[derive(Resource, Debug, PartialEq)]
pub struct AssetFolder<TAsset: Asset> {
	phantom_data: PhantomData<TAsset>,
	pub folder: Handle<LoadedFolder>,
}

impl<TAsset: Asset> AssetFolder<TAsset> {
	pub(crate) fn new(folder: Handle<LoadedFolder>) -> Self {
		Self {
			phantom_data: PhantomData,
			folder,
		}
	}
}

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
