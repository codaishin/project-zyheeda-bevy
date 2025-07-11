mod methods;
mod tools;
mod traits;

pub mod components;

use bevy::{ecs::query::QueryFilter, prelude::*};
use common::traits::{
	handles_map_generation::HandlesMapGeneration,
	handles_path_finding::HandlesPathFinding,
	register_derived_component::RegisterDerivedComponent,
	thread_safe::ThreadSafe,
};
use components::navigation::Navigation;
use methods::theta_star::ThetaStar;
use std::{collections::HashMap, marker::PhantomData};

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
	type TSystemSet = TMaps::TSystemSet;

	const SYSTEMS: Self::TSystemSet = TMaps::SYSTEMS;

	type TComputerRef = TMaps::TMapRef;

	fn computer_mapping_of<TFilter>() -> impl IntoSystem<(), HashMap<Entity, Self::TComputerRef>, ()>
	where
		TFilter: QueryFilter + 'static,
	{
		TMaps::map_mapping_of::<TFilter>()
	}
}
