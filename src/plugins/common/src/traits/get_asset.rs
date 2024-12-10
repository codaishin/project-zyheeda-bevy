use super::accessors::get::GetRefOption;
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
		TAssets: GetRefOption<Handle<Self::TAsset>, Self::TAsset>;
}
