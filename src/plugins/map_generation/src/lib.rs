mod cell_grid_size;
mod components;
mod errors;
mod grid_graph;
mod line_wide;
mod observers;
mod resources;
mod systems;
mod traits;

use crate::{
	components::{
		map::{Map, agents::AgentsLoaded, cells::corridor::Corridor, demo_map::DemoMap},
		map_agents::{AgentOfPersistentMap, GridAgentOf},
		wall_cell::WallCell,
		world_agent::WorldAgent,
	},
	resources::agents::color_lookup::{AgentsColorLookup, AgentsColorLookupImages},
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets},
	systems::log::OnError,
	traits::{
		handles_lights::HandlesLights,
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_map_generation::HandlesMapGeneration,
		handles_physics::{HandlesRaycast, colliders::HandlesColliders},
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		spawn::Spawn,
		thread_safe::ThreadSafe,
	},
};
use components::{floor_light::FloorLight, grid::Grid, wall_back::WallBack, wall_light::WallLight};
use grid_graph::GridGraph;
use std::marker::PhantomData;
use systems::{apply_extra_components::ApplyExtraComponents, unlit_material::unlit_material};
use traits::register_map_cell::RegisterMapCell;

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TLights, TPhysics>
	MapGenerationPlugin<(TLoading, TSavegame, TLights, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesColliders,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn from_plugins(_: &TLoading, _: &TSavegame, _: &TPhysics, _: &TLights) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TLights, TPhysics> Plugin
	for MapGenerationPlugin<(TLoading, TSavegame, TLights, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesColliders,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		let register_agents_lookup_load_tracking = TLoading::register_load_tracking::<
			AgentsColorLookup,
			LoadingEssentialAssets,
			AssetsProgress,
		>();
		register_agents_lookup_load_tracking.in_app(app, resource_exists::<AgentsColorLookup>);

		TSavegame::register_savable_component::<AgentsLoaded>(app);
		TSavegame::register_savable_component::<AgentOfPersistentMap>(app);
		TSavegame::register_savable_component::<DemoMap>(app);

		app.register_required_components::<Map, TSavegame::TSaveEntityMarker>()
			.register_required_components::<WallCell, TPhysics::TNoMouseHover>()
			.register_map_cell::<TLoading, TSavegame, Corridor>()
			.add_prefab_observer::<WallCell, TPhysics>()
			.add_systems(
				OnEnter(GameState::LoadingEssentialAssets),
				AgentsColorLookupImages::<Image>::lookup_images,
			)
			.add_systems(
				Update,
				AgentsColorLookup::parse_images
					.pipe(OnError::log)
					.run_if(not(resource_exists::<AgentsColorLookup>)),
			)
			.add_systems(OnEnter(GameState::NewGame), DemoMap::spawn)
			.add_systems(Update, Grid::<1>::insert.pipe(OnError::log))
			.add_systems(
				Update,
				(
					WallBack::apply_extra_components::<TLights>,
					WallLight::apply_extra_components::<TLights>,
					FloorLight::apply_extra_components::<TLights>,
					AgentOfPersistentMap::link_to_grid.run_if(in_state(GameState::Play)),
				)
					.in_set(Self::SYSTEMS),
			)
			.add_systems(Update, unlit_material);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct MapSystems;

impl<TDependencies> HandlesMapGeneration for MapGenerationPlugin<TDependencies> {
	type TMap = Grid<1>;
	type TGraph = GridGraph;
	type TSystemSet = MapSystems;
	type TNewWorldAgent = WorldAgent;

	const SYSTEMS: Self::TSystemSet = MapSystems;

	type TMapRef = GridAgentOf;
}
