use super::{get_asset::GetAsset, iteration::IterFinite};
use bevy::{ecs::query::QueryFilter, prelude::*};

pub trait RegisterAssetsForChildren {
	fn register_assets_for_children<TParent, TMarker>(app: &mut App)
	where
		TParent: Component
			+ ContainsAssetIdsForChildren<TMarker>
			+ GetAsset<TKey = TParent::TChildKey, TAsset = TParent::TChildAsset>,
		TParent::TKey: IterFinite,
		TMarker: Sync + Send + 'static;
}

pub trait ContainsAssetIdsForChildren<TMarker> {
	type TChildKey;
	type TChildFilter: QueryFilter;
	type TChildAsset: Asset;
	type TChildBundle: Bundle;

	fn child_name(key: &Self::TChildKey) -> &'static str;
	fn asset_component(asset: Option<&Self::TChildAsset>) -> Self::TChildBundle;
}
