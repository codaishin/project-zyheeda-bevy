mod components;
mod systems;

use crate::systems::apply_visualization::ApplyVisualization;
use bevy::prelude::*;
use common::{
	labels::Labels,
	systems::{
		insert_associated::{Configure, InsertAssociated, InsertOn},
		log::log_many,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		iteration::IterFinite,
		register_visualization::{ContainsVisibleItemAssets, RegisterVisualization},
	},
};
use components::visualize::Visualize;
use loading::{
	systems::is_processing::is_processing,
	traits::{
		progress::{AssetsProgress, DependenciesProgress},
		register_load_tracking::RegisterLoadTracking,
	},
};

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
	fn build(&self, _: &mut App) {}
}

impl RegisterVisualization for VisualizationPlugin {
	fn register_visualization<TComponent, TMarker>(app: &mut App)
	where
		TComponent: ContainsVisibleItemAssets<TMarker> + Component,
		TComponent::TKey: IterFinite,
		TMarker: Sync + Send + 'static,
	{
		app.register_load_tracking::<TMarker, DependenciesProgress>(
			Visualize::<TComponent, TMarker>::entities_loaded,
		)
		.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			InsertOn::<TComponent>::associated::<Visualize<TComponent, TMarker>>(
				Configure::LeaveAsIs,
			),
		)
		.add_systems(
			Update,
			(
				Visualize::<TComponent, TMarker>::track_in_self_and_children::<Name>()
					.filter::<TComponent::TVisualizationEntityConstraint>()
					.system(),
				TComponent::apply_visualization::<TMarker>
					.pipe(log_many)
					.run_if(not(is_processing::<AssetsProgress>))
					.run_if(not(is_processing::<DependenciesProgress>)),
			)
				.chain(),
		);
	}
}
