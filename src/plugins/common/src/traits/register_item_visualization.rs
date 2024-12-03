use super::{get_asset::GetAsset, iteration::IterFinite};
use bevy::{ecs::query::QueryFilter, prelude::*};

pub trait RegisterPlayerItemVisualization {
	fn register_player_item_visualization<TComponent, TView>(app: &mut App)
	where
		TComponent: ContainsVisibleItemAssets<TView> + Component,
		TComponent::TKey: IterFinite,
		TView: PlayerItemView;
}

pub trait ContainsVisibleItemAssets<TView>: GetAsset {
	type TVisualizationEntityConstraint: QueryFilter;

	fn visualization_entity_name(key: &Self::TKey) -> &'static str;
	fn visualization_component(item: Option<&Self::TAsset>) -> impl Bundle;
}

pub trait PlayerItemView: internal::PlayerItemView {}

impl<T> PlayerItemView for T where T: internal::PlayerItemView {}

pub struct Hand;
pub struct Forearm;
pub struct SubMeshEssence;

mod internal {
	use super::*;

	pub trait PlayerItemView {}

	impl PlayerItemView for Hand {}
	impl PlayerItemView for Forearm {}
	impl PlayerItemView for SubMeshEssence {}
}
