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
		handles_assets_for_children::{ChildAssetDefinition, HandlesAssetsForChildren},
		handles_load_tracking::{DependenciesProgress, HandlesLoadTracking, InApp},
		thread_safe::ThreadSafe,
	},
};
use components::children_lookup::ChildrenLookup;
use std::marker::PhantomData;

pub struct ChildrenAssetsDispatchPlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	pub fn depends_on(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	fn build(&self, _: &mut App) {}
}

impl<TLoading> HandlesAssetsForChildren for ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	fn register_child_asset<TParent, TMarker>(app: &mut App)
	where
		TParent: Component
			+ ChildAssetDefinition<TMarker>
			+ GetAsset<TKey = TParent::TChildKey, TAsset = TParent::TChildAsset>,
		TMarker: Sync + Send + 'static,
	{
		let all_children_present = ChildrenLookup::<TParent, TMarker>::entities_loaded;
		let dispatch_asset_components =
			TParent::dispatch_asset_components::<TMarker>.pipe(log_many);
		let insert_children_lookup =
			InsertOn::<TParent>::associated::<ChildrenLookup<TParent, TMarker>>;
		let store_children_in_lookup =
			ChildrenLookup::<TParent, TMarker>::track_in_self_and_children::<Name>()
				.filter::<TParent::TChildFilter>()
				.system();

		TLoading::register_load_tracking::<TMarker, DependenciesProgress>()
			.in_app(app, all_children_present);
		TLoading::register_after_load_system(app, Update, dispatch_asset_components);

		app.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			(
				insert_children_lookup(Configure::LeaveAsIs),
				store_children_in_lookup,
			)
				.chain(),
		);
	}
}
