use bevy::{asset::LoadedFolder, prelude::*};
use std::marker::PhantomData;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AssetFolder<TAsset: Asset> {
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
