mod components;
mod grid_graph;
mod line_wide;
mod map;
mod map_loader;
mod resources;
mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		handles_lights::HandlesLights,
		handles_map_generation::HandlesMapGeneration,
		prefab::RegisterPrefab,
		thread_safe::ThreadSafe,
	},
};
use components::{Wall, WallBack, level::Level};
use grid_graph::GridGraph;
use map::{LightCell, cell::MapCell};
use resources::load_level::LoadLevel;
use std::marker::PhantomData;
use systems::{
	apply_extra_components::ApplyExtraComponents,
	spawn_procedural::spawn_procedural,
	unlit_material::unlit_material,
};
use traits::{RegisterMapCell, light::wall::WallLight};

pub struct MapGenerationPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TPrefabs, TLights> MapGenerationPlugin<(TPrefabs, TLights)>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn depends_on(_: &TPrefabs, _: &TLights) -> Self {
		Self(PhantomData::<(TPrefabs, TLights)>)
	}
}

impl<TPrefabs, TLights> Plugin for MapGenerationPlugin<(TPrefabs, TLights)>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		let new_game = GameState::NewGame;

		app.register_map_cell::<MapCell>(OnEnter(new_game))
			.register_map_cell::<LightCell>(OnEnter(new_game))
			.add_systems(
				Update,
				(
					LoadLevel::<MapCell>::graph.pipe(Level::spawn::<MapCell>),
					LoadLevel::<LightCell>::graph.pipe(spawn_procedural),
				),
			)
			.add_systems(Update, Level::<2>::insert)
			.add_systems(
				Update,
				(
					Wall::apply_extra_components::<TLights>,
					WallBack::apply_extra_components::<TLights>,
					WallLight::apply_extra_components::<TLights>,
				),
			)
			.add_systems(Update, unlit_material);
	}
}

impl<TDependencies> HandlesMapGeneration for MapGenerationPlugin<TDependencies> {
	type TMap = Level<2>;
	type TGraph = GridGraph;
}
