mod components;
mod mesh_grid_graph;
mod observers;
mod resources;
mod system_params;
mod systems;

use crate::{
	components::{
		map::{
			Map,
			agents::AgentsLoaded,
			level::Level,
			objects::{MapObject, PersistentMapObject},
		},
		map_agents::{GridAgent, GridAgentOf},
		mesh_collider::MeshCollider,
		nav_mesh::NavMesh,
		spawner::Spawner,
		spawner_active::SpawnerActive,
	},
	mesh_grid_graph::MeshGridGraph,
	observers::identify_by_prefix::IdentifyByPrefix,
	resources::agents::prefab::PrefabRegister,
	system_params::set_agent_prefab::SetAgentPrefab,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		handles_enemies::EnemyType,
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_map_generation::{AgentType, HandlesMapGeneration, InteractiveType},
		handles_physics::{HandlesPhysicsConfig, HandlesRaycast},
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		spawn::Spawn,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::grid::Grid;
use std::marker::PhantomData;
use zyheeda_core::strings::normalized_name::NormalizedName;

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TPhysics> MapGenerationPlugin<(TLoading, TSavegame, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesPhysicsConfig,
{
	const AGENT_SPAWNERS: &[(GetNormalizedName, AgentType)] = &[
		(|| NormalizedName::from("PlayerSpawn"), AgentType::Player),
		(
			|| NormalizedName::from("VoidSphereSpawn"),
			AgentType::Enemy(EnemyType::VoidSphere),
		),
	];
	const INTERACTIVE_SPAWNERS: &[(GetNormalizedName, InteractiveType)] = &[(
		|| NormalizedName::from("SlideDoorSpawn"),
		InteractiveType::Door,
	)];
	const MESH_COLLIDER_PREFIX: &str = "Collider";
	const NAV_MESH_PREFIX: &str = "NavMesh";

	pub fn from_plugins(_: &TLoading, _: &TSavegame, _: &TPhysics) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TPhysics> Plugin for MapGenerationPlugin<(TLoading, TSavegame, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesPhysicsConfig,
{
	fn build(&self, app: &mut App) {
		TLoading::register_load_tracking::<Map, LoadingGame, AssetsProgress>()
			.in_app(app, Map::is_loaded);
		TLoading::register_load_tracking::<Spawner<AgentType>, LoadingGame, AssetsProgress>()
			.in_app(app, Spawner::<AgentType>::is_loaded);
		TLoading::register_load_tracking::<Spawner<InteractiveType>, LoadingGame, AssetsProgress>()
			.in_app(app, Spawner::<InteractiveType>::is_loaded);

		TSavegame::register_savable_component::<AgentsLoaded>(app);
		TSavegame::register_savable_component::<Map>(app);
		TSavegame::register_savable_component::<PersistentMapObject>(app);
		TSavegame::register_savable_component::<GridAgent>(app);
		TSavegame::register_savable_component::<Level<0>>(app);

		#[cfg(debug_assertions)]
		crate::mesh_grid_graph::debug::draw(app);

		app.init_resource::<PrefabRegister<AgentType>>()
			.init_resource::<PrefabRegister<InteractiveType>>()
			.add_systems(OnEnter(GameState::NewGame), Level::<0>::spawn)
			.add_prefab_observer::<MeshCollider, TPhysics::TConfigMut>()
			.add_observer(NavMesh::identify_by_prefix(Self::NAV_MESH_PREFIX))
			.add_observer(MeshCollider::identify_by_prefix(Self::MESH_COLLIDER_PREFIX))
			.add_observer(Spawner::<AgentType>::identify(Self::AGENT_SPAWNERS))
			.add_observer(Spawner::<InteractiveType>::identify(
				Self::INTERACTIVE_SPAWNERS,
			))
			.add_observer(SpawnerActive::remove_from_disabled_sources)
			.add_systems(
				Update,
				(
					NavMesh::spawn_grid::<MeshGridGraph>.pipe(OnError::log),
					MapObject::link_with_map.pipe(OnError::log),
					PersistentMapObject::link_with_map.pipe(OnError::log),
					Spawner::<AgentType>::execute,
					Spawner::<InteractiveType>::execute,
					Map::apply_map_persistence,
					GridAgent::link_to_grid::<MeshGridGraph>.run_if(in_state(GameState::Play)),
				)
					.chain(),
			);
	}
}

type GetNormalizedName = fn() -> NormalizedName;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct MapSystems;

impl<TDependencies> HandlesMapGeneration for MapGenerationPlugin<TDependencies> {
	type TMapPrefabs = SetAgentPrefab<'static>;

	type TGraph = MeshGridGraph;

	type TMap = Grid;
	type TMapRef = GridAgentOf;
}

impl<TDependencies> SystemSetDefinition for MapGenerationPlugin<TDependencies> {
	type TSystemSet = MapSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(MapSystems);
}
