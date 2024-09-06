use bevy::{
	asset::{Asset, Handle, LoadedFolder},
	prelude::Resource,
};
use std::{
	collections::HashSet,
	fmt::{Debug, Formatter, Result},
};

#[derive(Resource)]
pub(crate) struct SkillFolder<TFolder: Asset = LoadedFolder>(pub Handle<TFolder>);

impl<TFolder: Asset> Default for SkillFolder<TFolder> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<TFolder: Asset> Debug for SkillFolder<TFolder> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.debug_tuple("SkillFolder").field(&self.0).finish()
	}
}

impl<TFolder: Asset> PartialEq for SkillFolder<TFolder> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
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
