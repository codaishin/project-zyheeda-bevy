mod methods;
mod tools;
mod traits;

pub mod components;

use bevy::prelude::*;
use common::prelude::*;
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
	type TComputerRef = TMaps::TMapRef;
}

impl<TMaps> SystemSetDefinition for PathFindingPlugin<TMaps>
where
	TMaps: SystemSetDefinition + ThreadSafe,
{
	type TSystemSet = TMaps::TSystemSet;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = TMaps::SYSTEMS;
}
