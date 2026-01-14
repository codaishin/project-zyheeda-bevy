use crate::traits::asset_marker::AssetMarker;
use bevy::prelude::*;
use std::{any::TypeId, collections::HashMap};

#[derive(Component, Debug, Clone)]
/// Defines an asset that should be added to an [`Entity`]
///
/// This is a command like component and will be removed from
/// the [`Entity`] after the corresponding asset has been added.
pub struct InsertAsset<TAsset>
where
	TAsset: AssetMarker,
{
	new_asset: fn() -> TAsset,
	shared: Option<TypeId>,
}

impl<TAsset> PartialEq for InsertAsset<TAsset>
where
	TAsset: AssetMarker,
{
	fn eq(&self, other: &Self) -> bool {
		std::ptr::fn_addr_eq(self.new_asset, other.new_asset) && self.shared == other.shared
	}
}

impl<TAsset> InsertAsset<TAsset>
where
	TAsset: AssetMarker,
{
	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	pub fn unique(new_asset: fn() -> TAsset) -> Self {
		Self {
			new_asset,
			shared: None,
		}
	}

	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	///
	/// If a shared asset for `TDriver` already exists, that asset's handle will be used
	/// instead as input for [`AssetMarker::component`] and no new asset will be created.
	///
	/// `TDriver` can be any type, but is usually the component, which "drives" the insertion
	/// of this component, for instance via a `require` attribute.
	pub fn shared<TDriver>(new_asset: fn() -> TAsset) -> Self
	where
		TDriver: 'static,
	{
		Self {
			new_asset,
			shared: Some(TypeId::of::<TDriver>()),
		}
	}

	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	///
	/// If a shared asset for `type_id` already exists, that asset's handle will be used
	/// instead as input for [`AssetMarker::component`] and no new asset will be created.
	///
	/// `TypeId` can be any type, but is usually the id of a  component, which "drives" the insertion
	/// of this component, for instance via a `require` attribute.
	pub fn shared_id(new_asset: fn() -> TAsset, type_id: TypeId) -> Self {
		Self {
			new_asset,
			shared: Some(type_id),
		}
	}

	pub(crate) fn get_handle(
		&self,
		caches: &mut HashMap<TypeId, Handle<TAsset>>,
		assets: &mut Assets<TAsset>,
	) -> Handle<TAsset>
	where
		TAsset: AssetMarker,
	{
		let Some(shared_id) = self.shared else {
			return assets.add(self.create_asset());
		};

		let Some(handle) = caches.get(&shared_id) else {
			let handle = assets.add(self.create_asset());
			caches.insert(shared_id, handle.clone());
			return handle;
		};

		handle.clone()
	}

	pub(crate) fn create_asset(&self) -> TAsset {
		(self.new_asset)()
	}
}
