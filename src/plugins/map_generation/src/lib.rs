mod components;
mod grid_graph;
mod line_wide;
mod map_cells;
mod map_loader;
mod resources;
mod systems;
mod tools;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		handles_lights::HandlesLights,
		handles_map_generation::HandlesMapGeneration,
		thread_safe::ThreadSafe,
	},
};
use components::{floor_light::FloorLight, grid::Grid, wall_back::WallBack, wall_light::WallLight};
use grid_graph::GridGraph;
use map_cells::corridor::Corridor;
use std::marker::PhantomData;
use systems::{apply_extra_components::ApplyExtraComponents, unlit_material::unlit_material};
use traits::load_map::{LoadMap, LoadMapAsset};

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLights> MapGenerationPlugin<TLights>
where
	TLights: ThreadSafe + HandlesLights,
{
	pub fn from_plugin(_: &TLights) -> Self {
		Self(PhantomData::<TLights>)
	}
}

impl<TLights> Plugin for MapGenerationPlugin<TLights>
where
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		let new_game = GameState::NewGame;
		let loading = GameState::Loading;

		app.load_map_asset::<Corridor>(OnEnter(new_game))
			.load_map::<Corridor>(OnEnter(loading))
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
	type TGraph = GridGraph;
}
