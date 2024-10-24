pub mod components;
pub mod item;
pub mod traits;

use bevy::prelude::*;
use common::systems::{log::log_many, track_components::TrackComponentInSelfAndChildren};
use components::{visualize::VisualizeCommands, visualizer::Visualizer};
use traits::{entity_names::EntityNames, key_string::KeyString, view_component::ViewComponent};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
	fn build(&self, _: &mut App) {}
}

pub trait RegisterVisualizer<TKey> {
	fn register_view<TView, TConstraint>(&mut self) -> &mut Self
	where
		TView: EntityNames + KeyString<TKey> + ViewComponent + Send + Sync + 'static,
		TView::TViewComponent: Component + Clone + Default,
		TConstraint: Component;
}

impl<TKey> RegisterVisualizer<TKey> for App {
	fn register_view<TView, TConstraint>(&mut self) -> &mut Self
	where
		TView: EntityNames + KeyString<TKey> + ViewComponent + Send + Sync + 'static,
		TView::TViewComponent: Component + Clone + Default,
		TConstraint: Component,
	{
		self.add_systems(
			Update,
			(
				Visualizer::<TView>::track_in_self_and_children::<Name>()
					.with::<TConstraint>()
					.system(),
				VisualizeCommands::<TView>::apply.pipe(log_many),
			),
		)
	}
}
