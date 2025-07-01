mod components;
mod errors;
mod grid_graph;
mod line_wide;
mod map_cells;
mod map_loader;
mod observers;
mod systems;
mod tools;
mod traits;

use crate::components::{get_grid::GetGrid, map::demo_map::DemoMap};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		handles_lights::HandlesLights,
		handles_load_tracking::HandlesLoadTracking,
		handles_map_generation::HandlesMapGeneration,
		handles_saving::HandlesSaving,
		spawn::Spawn,
		thread_safe::ThreadSafe,
	},
};
use components::{floor_light::FloorLight, grid::Grid, wall_back::WallBack, wall_light::WallLight};
use grid_graph::GridGraph;
use map_cells::corridor::Corridor;
use std::marker::PhantomData;
use systems::{apply_extra_components::ApplyExtraComponents, unlit_material::unlit_material};
use traits::register_map_cell::RegisterMapCell;

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TLights> MapGenerationPlugin<(TLoading, TSavegame, TLights)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn from_plugins(_: &TLoading, _: &TSavegame, _: &TLights) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TLights> Plugin for MapGenerationPlugin<(TLoading, TSavegame, TLights)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		app.register_map_cell::<TLoading, TSavegame, Corridor>()
			.add_systems(OnEnter(GameState::NewGame), DemoMap::spawn)
			.add_systems(Update, Grid::<1>::insert)
			.add_systems(
				Update,
				(
					WallBack::apply_extra_components::<TLights>,
					WallLight::apply_extra_components::<TLights>,
					FloorLight::apply_extra_components::<TLights>,
				),
			)
			.add_systems(Update, unlit_material);
	}
}

impl<TDependencies> HandlesMapGeneration for MapGenerationPlugin<TDependencies> {
	type TMap = Grid<1>;
	type TMapAgent = GetGrid;
	type TGraph = GridGraph;
}
