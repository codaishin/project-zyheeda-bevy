use super::{get_asset::GetAsset, iteration::IterFinite};
use bevy::{ecs::query::QueryFilter, prelude::*};

pub trait RegisterVisualization {
	fn register_visualization<TComponent, TMarker>(app: &mut App)
	where
		TComponent: ContainsVisibleItemAssets<TMarker> + Component,
		TComponent::TKey: IterFinite,
		TMarker: Sync + Send + 'static;
}

pub trait ContainsVisibleItemAssets<TMarker>: GetAsset {
	type TVisualizationEntityConstraint: QueryFilter;

	fn visualization_entity_name(key: &Self::TKey) -> &'static str;
	fn visualization_component(item: Option<&Self::TAsset>) -> impl Bundle;
}
