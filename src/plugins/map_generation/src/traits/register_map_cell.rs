use super::{
	GridCellDistanceDefinition,
	insert_cell_components::InsertCellComponents,
	insert_cell_quadrant_components::InsertCellQuadrantComponents,
	is_walkable::IsWalkable,
};
use crate::{
	components::{
		grid::Grid,
		half_offset_grid::HalfOffsetGrid,
		map::{
			cells::{
				CellGrid,
				MapCells,
				agent::Agent,
				half_offset_cell::HalfOffsetCell,
				parsed_color::ParsedColor,
			},
			folder::MapFolder,
			grid_graph::MapGridGraph,
			image::MapImage,
		},
	},
	resources::map::color_lookup::{MapColorLookup, MapColorLookupImage},
	systems::map_color_lookup::load_images::ColorLookupAssetPath,
	traits::{map_cells_extra::MapCellsExtra, parse_map_image::ParseMapImage},
};
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	states::game_state::{GameState, LoadingEssentialAssets, LoadingGame},
	systems::log::OnError,
	traits::{
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			LoadTrackingInApp,
		},
		handles_saving::HandlesSaving,
		thread_safe::ThreadSafe,
	},
};

pub(crate) trait RegisterMapCell {
	fn register_map_cell<TLoading, TSavegame, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		TSavegame: ThreadSafe + HandlesSaving,
		for<'a> TCell: TypePath
			+ ParseMapImage<ParsedColor, TParseError = Unreachable, TLookup: Resource>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ ColorLookupAssetPath
			+ MapCellsExtra<TExtra = CellGrid<HalfOffsetCell<TCell>>>
			+ Default;
}

impl RegisterMapCell for App {
	fn register_map_cell<TLoading, TSavegame, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		for<'a> TCell: TypePath
			+ ParseMapImage<ParsedColor, TParseError = Unreachable, TLookup: Resource>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ ColorLookupAssetPath
			+ MapCellsExtra<TExtra = CellGrid<HalfOffsetCell<TCell>>>
			+ Default,
	{
		let resolving_dependencies =
			TLoading::processing_state::<LoadingGame, DependenciesProgress>();

		let register_map_lookup_load_tracking = TLoading::register_load_tracking::<
			MapColorLookup<TCell>,
			LoadingEssentialAssets,
			AssetsProgress,
		>();
		let register_map_images_load_tracking =
			TLoading::register_load_tracking::<MapImage<TCell>, LoadingGame, AssetsProgress>();
		let register_agent_images_load_tracking = TLoading::register_load_tracking::<
			MapImage<Agent<TCell>>,
			LoadingGame,
			AssetsProgress,
		>();

		// Track wether assets have been loaded
		register_map_lookup_load_tracking.in_app(self, resource_exists::<MapColorLookup<TCell>>);
		register_map_images_load_tracking.in_app(self, MapImage::<TCell>::all_loaded);
		register_agent_images_load_tracking.in_app(self, MapImage::<Agent<TCell>>::all_loaded);

		self
			// Map color lookup
			.add_systems(
				OnEnter(GameState::LoadingEssentialAssets),
				MapColorLookupImage::<TCell>::lookup_images,
			)
			.add_systems(
				Update,
				MapColorLookup::<TCell>::parse_images
					.pipe(OnError::log)
					.run_if(not(resource_exists::<MapColorLookup<TCell>>)),
			)
			// Load map cells and root graph from image
			.add_observer(MapFolder::<TCell>::load_map_image("map.png"))
			.add_observer(MapFolder::<Agent<TCell>>::load_map_image("agents.png"))
			.add_systems(Update, MapFolder::<TCell>::load_agents)
			.add_systems(
				OnEnter(resolving_dependencies),
				(
					MapImage::<TCell>::insert_map_cells.pipe(OnError::log),
					MapImage::<Agent<TCell>>::insert_map_cells.pipe(OnError::log),
					MapCells::<TCell>::insert_map_grid_graph,
					MapCells::<Agent<TCell>>::spawn_map_agents,
				)
					.chain(),
			)
			// Generate child grid for navigation
			.add_observer(MapGridGraph::<TCell>::spawn_child::<Grid>)
			.add_observer(
				Grid::compute_cells::<TCell>
					.pipe(Grid::spawn_cells)
					.pipe(OnError::log),
			)
			// Generate child grid with 1/2 offset for map models
			.add_observer(MapGridGraph::<TCell>::spawn_child::<HalfOffsetGrid>)
			.add_observer(
				HalfOffsetGrid::compute_cells::<TCell>
					.pipe(HalfOffsetGrid::spawn_cells)
					.pipe(OnError::log),
			)
	}
}
