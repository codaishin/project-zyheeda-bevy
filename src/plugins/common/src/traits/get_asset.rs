use super::accessors::get::GetRef;
use bevy::prelude::*;

pub trait GetAsset {
	type TKey;
	type TAsset: Asset;

	fn get_asset<'a, TAssets>(
		&'a self,
		key: &Self::TKey,
		assets: &'a TAssets,
	) -> Option<&'a Self::TAsset>
	where
		TAssets: GetRef<Handle<Self::TAsset>, TValue<'a> = &'a Self::TAsset>;
}
