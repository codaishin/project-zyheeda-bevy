mod methods;
mod tools;
mod traits;

pub mod components;

use bevy::prelude::*;
use common::{
	systems::log::log_many,
	traits::{
		handles_map_generation::HandlesMapGeneration,
		handles_path_finding::HandlesPathFinding,
		thread_safe::ThreadSafe,
	},
};
use components::nav_grid::NavGrid;
use methods::theta_star::ThetaStar;
use std::marker::PhantomData;

type TNavGrid = NavGrid<ThetaStar>;

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
		app.register_required_components::<TMaps::TMap, TNavGrid>();
		app.add_systems(Update, TNavGrid::update_from::<TMaps::TMap>.pipe(log_many));
	}
}

impl<TDependencies> HandlesPathFinding for PathFindingPlugin<TDependencies> {
	type TComputePath = TNavGrid;
}
