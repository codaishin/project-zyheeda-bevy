mod methods;
mod tools;
mod traits;

pub mod components;

use bevy::prelude::*;
use common::{
	labels::Labels,
	systems::insert_required::{InsertOn, InsertRequired},
	traits::{
		handles_map_generation::HandlesMapGeneration,
		handles_path_finding::HandlesPathFinding,
		thread_safe::ThreadSafe,
	},
};
use components::navigation::Navigation;
use methods::theta_star::ThetaStar;
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
		app.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			InsertOn::<TMaps::TMap>::required(|map| Navigation {
				graph: TMaps::TGraph::from(map),
				method: ThetaStar::default(),
			}),
		);
	}
}

impl<TMaps> HandlesPathFinding for PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	type TComputePath = Navigation<ThetaStar, TMaps::TGraph>;
}
