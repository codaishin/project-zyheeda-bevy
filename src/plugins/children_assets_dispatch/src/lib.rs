mod components;
mod systems;

use crate::systems::dispatch_asset_components::DispatchAssetComponents;
use bevy::prelude::*;
use common::{
	labels::Labels,
	systems::{
		insert_associated::{Configure, InsertAssociated, InsertOn},
		log::log_many,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		get_asset::GetAsset,
		iteration::IterFinite,
		register_assets_for_children::{ContainsAssetIdsForChildren, RegisterAssetsForChildren},
	},
};
use components::children_lookup::ChildrenLookup;
use loading::{
	systems::is_processing::is_processing,
	traits::{
		progress::{AssetsProgress, DependenciesProgress},
		register_load_tracking::RegisterLoadTracking,
	},
};

pub struct ChildrenAssetsDispatchPlugin;

impl Plugin for ChildrenAssetsDispatchPlugin {
	fn build(&self, _: &mut App) {}
}

impl RegisterAssetsForChildren for ChildrenAssetsDispatchPlugin {
	fn register_assets_for_children<TParent, TMarker>(app: &mut App)
	where
		TParent: Component
			+ ContainsAssetIdsForChildren<TMarker>
			+ GetAsset<TKey = TParent::TChildKey, TAsset = TParent::TChildAsset>,
		TParent::TChildKey: IterFinite,
		TMarker: Sync + Send + 'static,
	{
		let on_prefab_instantiation = Labels::PREFAB_INSTANTIATION.label();
		let all_children_present = ChildrenLookup::<TParent, TMarker>::entities_loaded;
		let insert_children_lookup =
			InsertOn::<TParent>::associated::<ChildrenLookup<TParent, TMarker>>;
		let store_children_in_lookup =
			ChildrenLookup::<TParent, TMarker>::track_in_self_and_children::<Name>()
				.filter::<TParent::TChildFilter>()
				.system();
		let dispatch_asset_components_to_children = TParent::dispatch_asset_components::<TMarker>
			.pipe(log_many)
			.run_if(not(is_processing::<AssetsProgress>))
			.run_if(not(is_processing::<DependenciesProgress>));

		app.register_load_tracking::<TMarker, DependenciesProgress>(all_children_present)
			.add_systems(
				on_prefab_instantiation,
				insert_children_lookup(Configure::LeaveAsIs),
			)
			.add_systems(
				Update,
				(
					store_children_in_lookup,
					dispatch_asset_components_to_children,
				)
					.chain(),
			);
	}
}
