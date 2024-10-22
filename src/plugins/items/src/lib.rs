pub mod components;
pub mod item;
pub mod traits;

use bevy::prelude::*;
use common::systems::{log::log_many, track_components::TrackComponentInSelfAndChildren};
use components::{visualize::Visualize, visualizer::Visualizer};
use traits::{entity_names::EntityNames, key_string::KeyString};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
	fn build(&self, _: &mut App) {}
}

pub trait RegisterVisualizer<TKey> {
	fn register_visualizer<TVisualizer, TConstraint: Component>(&mut self) -> &mut Self
	where
		TVisualizer: EntityNames + KeyString<TKey> + Send + Sync + 'static;
}

impl<TKey> RegisterVisualizer<TKey> for App {
	fn register_visualizer<TVisualizer, TConstraint: Component>(&mut self) -> &mut Self
	where
		TVisualizer: EntityNames + KeyString<TKey> + Send + Sync + 'static,
	{
		self.add_systems(
			Update,
			(
				Visualizer::<TVisualizer>::track_in_self_and_children::<Name>()
					.with::<TConstraint>()
					.system(),
				Visualize::<TVisualizer>::system.pipe(log_many),
			),
		)
	}
}
