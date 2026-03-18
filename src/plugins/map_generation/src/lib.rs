mod components;
mod mesh_grid_graph;
mod observers;
mod resources;
mod system_params;
mod systems;

use crate::{
	components::{
		agent_spawner::{AgentSpawner, SpawnerActive},
		map::{
			Map,
			agents::AgentsLoaded,
			bay::BayMap,
			objects::{MapObject, PersistentMapObject},
		},
		map_agents::{GridAgent, GridAgentOf},
		mesh_collider::MeshCollider,
		nav_mesh::NavMesh,
	},
	mesh_grid_graph::MeshGridGraph,
	observers::identify_by_prefix::IdentifyByPrefix,
	resources::agents::prefab::AgentPrefab,
	system_params::set_agent_prefab::SetAgentPrefab,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	traits::{
		handles_enemies::EnemyType,
		handles_lights::HandlesLights,
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_map_generation::{AgentType, HandlesMapGeneration},
		handles_physics::{HandlesRaycast, physical_bodies::HandlesPhysicalBodies},
		handles_saving::HandlesSaving,
		spawn::Spawn,
		thread_safe::ThreadSafe,
	},
};
use components::grid::Grid;
use std::marker::PhantomData;

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
		TLoading::register_load_tracking::<Map, LoadingGame, AssetsProgress>()
			.in_app(app, Map::is_loaded);
		TLoading::register_load_tracking::<AgentSpawner, LoadingGame, AssetsProgress>()
			.in_app(app, AgentSpawner::is_loaded);

		TSavegame::register_savable_component::<AgentsLoaded>(app);
		TSavegame::register_savable_component::<Map>(app);
		TSavegame::register_savable_component::<BayMap>(app);
		TSavegame::register_savable_component::<PersistentMapObject>(app);
		TSavegame::register_savable_component::<GridAgent>(app);

		#[cfg(debug_assertions)]
		crate::mesh_grid_graph::debug::draw(app);

		app.init_resource::<AgentPrefab>()
			.register_required_components::<Map, TSavegame::TSaveEntityMarker>()
			.register_required_components_with::<MeshCollider, TPhysics::TBody>(MeshCollider::body)
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
					GridAgent::link_to_grid::<MeshGridGraph>.run_if(in_state(GameState::Play)),
				)
					.chain(),
			);
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
