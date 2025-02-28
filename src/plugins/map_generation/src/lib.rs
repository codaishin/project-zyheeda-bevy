mod components;
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
use components::{level::Level, Wall, WallBack};
use map::{cell::MapCell, LightCell};
use resources::load_level::LoadLevel;
use std::marker::PhantomData;
use systems::{
	apply_extra_components::ApplyExtraComponents,
	spawn_procedural::spawn_procedural,
	unlit_material::unlit_material,
};
use traits::{light::wall::WallLight, RegisterMapCell};

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
					LoadLevel::<MapCell>::cell_transforms.pipe(Level::spawn::<MapCell>),
					LoadLevel::<LightCell>::cell_transforms.pipe(spawn_procedural),
				),
			)
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
	type TMap = Level;
}
