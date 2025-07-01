mod methods;
mod tools;
mod traits;

pub mod components;

use bevy::prelude::*;
use common::traits::{
	handles_map_generation::HandlesMapGeneration,
	handles_path_finding::HandlesPathFinding,
	register_derived_component::RegisterDerivedComponent,
	thread_safe::ThreadSafe,
};
use components::navigation::Navigation;
use methods::theta_star::ThetaStar;
use std::marker::PhantomData;

pub struct PathFindingPlugin<TMap>(PhantomData<TMap>);

impl<TMaps> PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	pub fn from_plugin(_: &TMaps) -> Self {
		Self(PhantomData)
	}
}

impl<TMaps> Plugin for PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	fn build(&self, app: &mut App) {
		app.register_derived_component::<TMaps::TMap, Navigation<ThetaStar, TMaps::TGraph>>();
	}
}

impl<TMaps> HandlesPathFinding for PathFindingPlugin<TMaps>
where
	TMaps: HandlesMapGeneration + ThreadSafe,
{
	type TComputePath = Navigation<ThetaStar, TMaps::TGraph>;
	type TPathAgent = TMaps::TMapAgent;
}
