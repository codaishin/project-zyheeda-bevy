pub mod components;
pub mod item;
pub mod traits;

use bevy::prelude::*;
use common::{
	systems::{
		insert_associated::{Configure, InsertAssociated, InsertOn},
		log::log_many,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::iteration::IterFinite,
};
use components::{visualize::VisualizeCommands, visualizer::Visualizer};
use loading::{
	systems::is_processing::is_processing,
	traits::{
		progress::{AssetsProgress, DependenciesProgress},
		register_load_tracking::RegisterLoadTracking,
	},
};
use traits::view::ItemView;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
	fn build(&self, _: &mut App) {}
}

pub trait RegisterItemView<TKey> {
	fn register_item_view_for<TAgent, TView>(&mut self) -> &mut Self
	where
		TAgent: Component,
		TView: ItemView<TKey> + Send + Sync + 'static,
		TKey: IterFinite + Sync + Send + 'static;
}

impl<TKey> RegisterItemView<TKey> for App {
	fn register_item_view_for<TAgent, TView>(&mut self) -> &mut Self
	where
		TAgent: Component,
		TView: ItemView<TKey> + Send + Sync + 'static,
		TKey: IterFinite + Sync + Send + 'static,
	{
		self.register_load_tracking::<TView, DependenciesProgress>(
			Visualizer::<TView, TKey>::view_entities_loaded,
		)
		.add_systems(
			PreUpdate,
			InsertOn::<TAgent>::associated::<Visualizer<TView, TKey>>(Configure::LeaveAsIs),
		)
		.add_systems(
			Update,
			(
				Visualizer::<TView, TKey>::track_in_self_and_children::<Name>()
					.filter::<TView::TFilter>()
					.system(),
				VisualizeCommands::<TView, TKey>::apply
					.pipe(log_many)
					.run_if(not(is_processing::<AssetsProgress>))
					.run_if(not(is_processing::<DependenciesProgress>)),
			)
				.chain(),
		)
	}
}
