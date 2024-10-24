pub mod components;
pub mod item;
pub mod traits;

use bevy::{ecs::query::QueryFilter, prelude::*};
use common::systems::{log::log_many, track_components::TrackComponentInSelfAndChildren};
use components::{visualize::VisualizeCommands, visualizer::Visualizer};
use traits::{
	entity_names::EntityNames,
	key_string::KeyString,
	view_component::ViewComponent,
	view_filter::ViewFilter,
};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
	fn build(&self, _: &mut App) {}
}

pub trait RegisterItemView<TKey> {
	fn register_item_view<TView>(&mut self) -> &mut Self
	where
		TView: EntityNames + KeyString<TKey> + ViewComponent + ViewFilter + Send + Sync + 'static,
		TView::TViewComponent: Component + Clone + Default,
		TView::TFilter: QueryFilter + 'static;
}

impl<TKey> RegisterItemView<TKey> for App {
	fn register_item_view<TView>(&mut self) -> &mut Self
	where
		TView: EntityNames + KeyString<TKey> + ViewComponent + ViewFilter + Send + Sync + 'static,
		TView::TViewComponent: Component + Clone + Default,
		TView::TFilter: QueryFilter + 'static,
	{
		self.add_systems(
			Update,
			(
				Visualizer::<TView>::track_in_self_and_children::<Name>()
					.filter::<TView::TFilter>()
					.system(),
				VisualizeCommands::<TView>::apply.pipe(log_many),
			),
		)
	}
}
