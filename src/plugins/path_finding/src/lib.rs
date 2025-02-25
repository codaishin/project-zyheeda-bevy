mod components;

use bevy::prelude::*;
use common::traits::{handles_map_generation::HandlesMapGeneration, thread_safe::ThreadSafe};
use components::nav_grid::NavGrid;
use std::marker::PhantomData;

pub struct PathFindingPlugin<TMap>(PhantomData<TMap>);

impl<TMaps> PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	pub fn depends_on(_: &TMaps) -> Self {
		Self(PhantomData)
	}
}

impl<TMaps> Plugin for PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	fn build(&self, app: &mut App) {
		app.register_required_components::<TMaps::TMap, NavGrid>();
		app.add_systems(Update, NavGrid::update_from::<TMaps::TMap>);
	}
}
