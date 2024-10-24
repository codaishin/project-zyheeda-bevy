pub mod components;
pub mod item;
pub mod traits;

use bevy::prelude::*;
use common::{
	systems::{
		add_component_to::AddTo,
		log::log_many,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::iteration::IterFinite,
};
use components::{visualize::VisualizeCommands, visualizer::Visualizer};
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
		self.add_systems(PreUpdate, Visualizer::<TView, TKey>::add_to::<TAgent>)
			.add_systems(
				Update,
				(
					Visualizer::<TView, TKey>::track_in_self_and_children::<Name>()
						.filter::<TView::TFilter>()
						.system(),
					VisualizeCommands::<TView, TKey>::apply.pipe(log_many),
				),
			)
	}
}
