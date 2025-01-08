use super::{get_asset::GetAsset, iteration::IterFinite};
use bevy::{ecs::query::QueryFilter, prelude::*};

/// Handles attachment of additional asset components to children
/// defined in [`ChildAssetDefinition`]
pub trait HandlesAssetsForChildren {
	fn register_child_asset<TParent, TMarker>(app: &mut App)
	where
		TParent: Component
			+ ChildAssetDefinition<TMarker>
			+ GetAsset<TKey = TParent::TChildKey, TAsset = TParent::TChildAsset>,
		TMarker: Sync + Send + 'static;
}

/// Defines assets that should be attached as components to child entities.
/// `TMarker` allows implementation of different versions of this
/// trait for the same type.
pub trait ChildAssetDefinition<TMarker> {
	/// Identifies valid target children by name
	type TChildKey: IterFinite + ChildName<TMarker>;

	/// Additional filter for children to identify valid children.
	type TChildFilter: QueryFilter;

	/// The asset that should be attached.
	type TChildAsset: Asset + ChildAssetComponent<TMarker>;
}

pub trait ChildName<TMarker> {
	fn child_name(&self) -> &'static str;
}

pub trait ChildAssetComponent<TMarker> {
	type TComponent: Component;

	fn component(asset: Option<&Self>) -> Self::TComponent;
}
