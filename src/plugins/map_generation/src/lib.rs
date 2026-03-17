mod cell_grid_size;
mod components;
mod errors;
mod line_wide;
mod mesh_grid_graph;
mod observers;
mod resources;
mod square_grid_graph;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::{
		agent_spawner::{AgentSpawner, SpawnerActive},
		map::{
			Map,
			agents::AgentsLoaded,
			bay::BayMap,
			cells::corridor::Corridor,
			demo_map::DemoMap,
			objects::{MapObject, PersistentMapObject},
		},
		map_agents::{GridAgent, GridAgentOf},
		mesh_collider::MeshCollider,
		nav_mesh::NavMesh,
		wall_cell::WallCell,
	},
	mesh_grid_graph::MeshGridGraph,
	observers::identify_by_prefix::IdentifyByPrefix,
	resources::agents::{
		color_lookup::{AgentsColorLookup, AgentsColorLookupImages},
		prefab::AgentPrefab,
	},
	square_grid_graph::SquareGridGraph,
	system_params::set_agent_prefab::SetAgentPrefab,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets, LoadingGame},
	systems::log::OnError,
	traits::{
		handles_enemies::EnemyType,
		handles_lights::HandlesLights,
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_map_generation::{AgentType, HandlesMapGeneration},
		handles_physics::{HandlesRaycast, physical_bodies::HandlesPhysicalBodies},
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		spawn::Spawn,
		thread_safe::ThreadSafe,
	},
};
use components::{floor_light::FloorLight, grid::Grid, wall_back::WallBack, wall_light::WallLight};
use std::marker::PhantomData;
use systems::{apply_extra_components::ApplyExtraComponents, unlit_material::unlit_material};
use traits::register_map_cell::RegisterMapCell;

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TLights, TPhysics>
	MapGenerationPlugin<(TLoading, TSavegame, TLights, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesPhysicalBodies,
	TLights: ThreadSafe + HandlesLights,
{
	const SPAWNERS: &[(&str, AgentType)] = &[
		("PlayerSpawn", AgentType::Player),
		("VoidSphereSpawn", AgentType::Enemy(EnemyType::VoidSphere)),
	];
	const MESH_COLLIDER_PREFIX: &str = "Collider";
	const NAV_MESH_PREFIX: &str = "NavMesh";

	pub fn from_plugins(_: &TLoading, _: &TSavegame, _: &TPhysics, _: &TLights) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TLights, TPhysics> Plugin
	for MapGenerationPlugin<(TLoading, TSavegame, TLights, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesPhysicalBodies,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		let register_agents_lookup_load_tracking = TLoading::register_load_tracking::<
			AgentsColorLookup,
			LoadingEssentialAssets,
			AssetsProgress,
		>();
		register_agents_lookup_load_tracking.in_app(app, resource_exists::<AgentsColorLookup>);

		TLoading::register_load_tracking::<Map, LoadingGame, AssetsProgress>()
			.in_app(app, Map::is_loaded);
		TLoading::register_load_tracking::<AgentSpawner, LoadingGame, AssetsProgress>()
			.in_app(app, AgentSpawner::is_loaded);

		TSavegame::register_savable_component::<AgentsLoaded>(app);
		TSavegame::register_savable_component::<Map>(app);
		TSavegame::register_savable_component::<BayMap>(app);
		TSavegame::register_savable_component::<DemoMap>(app);
		TSavegame::register_savable_component::<PersistentMapObject>(app);
		TSavegame::register_savable_component::<GridAgent>(app);

		#[cfg(debug_assertions)]
		crate::mesh_grid_graph::debug::draw(app);

		app.init_resource::<AgentPrefab>()
			.register_required_components::<Map, TSavegame::TSaveEntityMarker>()
			.register_required_components_with::<MeshCollider, TPhysics::TBody>(MeshCollider::body)
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
			.add_systems(OnEnter(GameState::NewGame), BayMap::spawn)
			.add_observer(NavMesh::identify_by_prefix(Self::NAV_MESH_PREFIX))
			.add_observer(MeshCollider::identify_by_prefix(Self::MESH_COLLIDER_PREFIX))
			.add_observer(AgentSpawner::identify_by_prefix_map(Self::SPAWNERS))
			.add_observer(SpawnerActive::remove_when_map_created_from_save)
			.add_systems(
				Update,
				(
					MapObject::link_with_map.pipe(OnError::log),
					PersistentMapObject::link_with_map.pipe(OnError::log),
					NavMesh::spawn_grid::<MeshGridGraph>.pipe(OnError::log),
					AgentSpawner::spawn_agent,
					GridAgent::link_to_grid::<SquareGridGraph>.run_if(in_state(GameState::Play)),
					GridAgent::link_to_grid::<MeshGridGraph>.run_if(in_state(GameState::Play)),
				)
					.chain(),
			)
			.add_systems(Update, Grid::<1>::insert.pipe(OnError::log))
			.add_systems(
				Update,
				(
					WallBack::apply_extra_components::<TLights>,
					WallLight::apply_extra_components::<TLights>,
					FloorLight::apply_extra_components::<TLights>,
				)
					.in_set(Self::SYSTEMS),
			)
			.add_systems(Update, unlit_material);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct MapSystems;

impl<TDependencies> HandlesMapGeneration for MapGenerationPlugin<TDependencies> {
	const SYSTEMS: Self::TSystemSet = MapSystems;
	type TSystemSet = MapSystems;

	type TNewMapAgent<'w, 's> = SetAgentPrefab<'w>;

	type TGraph = MeshGridGraph;

	type TMap = Grid<0, MeshGridGraph>;
	type TMapRef = GridAgentOf;
}
