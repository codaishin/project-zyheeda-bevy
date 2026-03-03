mod components;
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
		agent_spawner::AgentSpawner,
		map::{Map, bay::BayMap},
		map_agents::{AgentOfPersistentMap, GridAgentOf},
		mesh_collider::MeshCollider,
	},
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
use components::{floor_light::FloorLight, grid::Grid, wall_back::WallBack, wall_light::WallLight};
use square_grid_graph::SquareGridGraph;
use std::marker::PhantomData;
use systems::{apply_extra_components::ApplyExtraComponents, unlit_material::unlit_material};

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TLights, TPhysics>
	MapGenerationPlugin<(TLoading, TSavegame, TLights, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesRaycast + HandlesPhysicalBodies,
	TLights: ThreadSafe + HandlesLights,
{
	const SPAWNERS: &[(&'static str, AgentType)] = &[
		("PlayerSpawn", AgentType::Player),
		("VoidSphereSpawn", AgentType::Enemy(EnemyType::VoidSphere)),
	];

	const MESH_COLLIDER_PREFIX: &str = "Collider";

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

		TSavegame::register_savable_component::<AgentOfPersistentMap>(app);
		TSavegame::register_savable_component::<Map>(app);
		TSavegame::register_savable_component::<BayMap>(app);

		app.init_resource::<AgentPrefab>()
			.register_required_components::<Map, TSavegame::TSaveEntityMarker>()
			.register_required_components_with::<MeshCollider, TPhysics::TBody>(MeshCollider::body)
			.add_systems(OnEnter(GameState::NewGame), BayMap::spawn)
			.add_observer(AgentSpawner::identify(Self::SPAWNERS))
			.add_observer(MeshCollider::identify(Self::MESH_COLLIDER_PREFIX))
			.add_systems(
				Update,
				(
					AgentSpawner::link_with_map.pipe(OnError::log),
					AgentSpawner::spawn_agent,
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
	const SYSTEMS: Self::TSystemSet = MapSystems;
	type TSystemSet = MapSystems;

	type TNewMapAgent<'w, 's> = SetAgentPrefab<'w>;

	type TGraph = SquareGridGraph;

	type TMap = Grid<1>;
	type TMapRef = GridAgentOf;
}
