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
			cells::{MapCells, parsed_color::ParsedColor},
			folder::MapFolder,
			grid_graph::MapGridGraph,
			image::MapImage,
		},
	},
	resources::map::color_lookup::{MapColorLookup, MapColorLookupImage},
	systems::map_color_lookup::load_images::ColorLookupAssetPath,
	traits::parse_map_image::ParseMapImage,
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
			+ ParseMapImage<ParsedColor, TCell, TParseError = Unreachable>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ ColorLookupAssetPath
			+ Default;
}

impl RegisterMapCell for App {
	fn register_map_cell<TLoading, TSavegame, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		TSavegame: ThreadSafe + HandlesSaving,
		for<'a> TCell: TypePath
			+ ParseMapImage<ParsedColor, TCell, TParseError = Unreachable>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ ColorLookupAssetPath
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

		//save maps
		TSavegame::register_savable_component::<MapFolder<TCell>>(self);
		self.register_required_components::<MapFolder<TCell>, TSavegame::TSaveEntityMarker>();

		// Track wether assets have been loaded
		register_map_lookup_load_tracking.in_app(self, resource_exists::<MapColorLookup<TCell>>);
		register_map_images_load_tracking.in_app(self, MapImage::<TCell>::all_loaded);

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
			.add_observer(MapFolder::<TCell>::load_map_image)
			.add_systems(
				OnEnter(resolving_dependencies),
				(
					MapImage::<TCell>::insert_map_cells.pipe(OnError::log),
					MapCells::<TCell>::insert_map_grid_graph.pipe(OnError::log),
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
