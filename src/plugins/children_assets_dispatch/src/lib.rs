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
		register_load_tracking::{DependenciesProgress, InApp, RegisterLoadTracking},
	},
};
use components::children_lookup::ChildrenLookup;
use std::marker::PhantomData;

pub struct ChildrenAssetsDispatchPlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: Plugin + RegisterLoadTracking,
{
	pub fn depends_on(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: Plugin + RegisterLoadTracking,
{
	fn build(&self, _: &mut App) {}
}

impl<TLoading> RegisterAssetsForChildren for ChildrenAssetsDispatchPlugin<TLoading>
where
	TLoading: Plugin + RegisterLoadTracking,
{
	fn register_assets_for_children<TParent, T>(app: &mut App)
	where
		TParent: Component
			+ ContainsAssetIdsForChildren<T>
			+ GetAsset<TKey = TParent::TChildKey, TAsset = TParent::TChildAsset>,
		TParent::TChildKey: IterFinite,
		T: Sync + Send + 'static,
	{
		let on_prefab_instantiation = Labels::PREFAB_INSTANTIATION.label();
		let all_children_present = ChildrenLookup::<TParent, T>::entities_loaded;
		let insert_children_lookup = InsertOn::<TParent>::associated::<ChildrenLookup<TParent, T>>;
		let store_children_in_lookup =
			ChildrenLookup::<TParent, T>::track_in_self_and_children::<Name>()
				.filter::<TParent::TChildFilter>()
				.system();
		let dispatch_asset_components_to_children =
			TParent::dispatch_asset_components::<T>.pipe(log_many);

		TLoading::register_after_load_system(app, Update, dispatch_asset_components_to_children);
		TLoading::register_load_tracking::<T, DependenciesProgress>()
			.in_app(app, all_children_present);

		app.add_systems(
			on_prefab_instantiation,
			(
				insert_children_lookup(Configure::LeaveAsIs),
				store_children_in_lookup,
			)
				.chain(),
		);
	}
}
