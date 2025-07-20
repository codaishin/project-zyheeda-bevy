mod components;
mod errors;
mod grid_graph;
mod line_wide;
mod observers;
mod resources;
mod systems;
mod traits;

use crate::{
	components::map::{
		Map,
		agents::{AgentsLoaded, Enemy, Player},
		cells::corridor::Corridor,
		demo_map::DemoMap,
	},
	resources::agents::color_lookup::{AgentsColorLookup, AgentsColorLookupImages},
	systems::get_grid::EntityOfGrid,
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_rapier3d::prelude::Collider;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets},
	systems::log::OnError,
	traits::{
		handles_enemies::{HandlesEnemies, HandlesEnemyBehaviors},
		handles_lights::HandlesLights,
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_map_generation::{EntityMapFiltered, HandlesMapGeneration},
		handles_player::HandlesPlayer,
		handles_saving::HandlesSaving,
		register_derived_component::RegisterDerivedComponent,
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

impl<TLoading, TSavegame, TLights, TPlayer, TEnemies>
	MapGenerationPlugin<(TLoading, TSavegame, TLights, TPlayer, TEnemies)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TLights: ThreadSafe + HandlesLights,
	TPlayer: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemyBehaviors,
{
	pub fn from_plugins(
		_: &TLoading,
		_: &TSavegame,
		_: &TLights,
		_: &TPlayer,
		_: &TEnemies,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TLights, TPlayer, TEnemies> Plugin
	for MapGenerationPlugin<(TLoading, TSavegame, TLights, TPlayer, TEnemies)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TLights: ThreadSafe + HandlesLights,
	TPlayer: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemies,
{
	fn build(&self, app: &mut App) {
		let register_agents_lookup_load_tracking = TLoading::register_load_tracking::<
			AgentsColorLookup,
			LoadingEssentialAssets,
			AssetsProgress,
		>();
		register_agents_lookup_load_tracking.in_app(app, resource_exists::<AgentsColorLookup>);

		TSavegame::register_savable_component::<AgentsLoaded>(app);
		TSavegame::register_savable_component::<DemoMap>(app);

		app.register_required_components::<Map, TSavegame::TSaveEntityMarker>()
			.register_required_components::<Player, TPlayer::TPlayer>()
			.register_required_components::<Enemy, TEnemies::TEnemy>()
			.register_derived_component::<Grid, Collider>()
			.register_map_cell::<TLoading, TSavegame, Corridor>()
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
			.add_systems(Update, Grid::<1>::insert)
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
	type TMap = Grid<1>;
	type TGraph = GridGraph;
	type TSystemSet = MapSystems;

	const SYSTEMS: Self::TSystemSet = MapSystems;

	type TMapRef = EntityOfGrid;

	fn map_mapping_of<TFilter>()
	-> impl IntoSystem<(), EntityMapFiltered<Self::TMapRef, TFilter>, ()>
	where
		TFilter: QueryFilter + 'static,
	{
		IntoSystem::into_system(
			EntityOfGrid::get_grid::<TFilter>
				.pipe(OnError::log_and_return(EntityMapFiltered::default)),
		)
	}
}
