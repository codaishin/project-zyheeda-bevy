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
			asset::MapAsset,
			cells::{MapCells, parsed_color::ParsedColor},
			grid_graph::MapGridGraph,
			image::MapImage,
		},
	},
	resources::color_lookup::{ColorLookup, ColorLookupImage},
	systems::color_lookup::load_images::ColorLookupAssetPath,
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

		let map_lookup_progress = TLoading::register_load_tracking::<
			ColorLookup<TCell>,
			LoadingEssentialAssets,
			AssetsProgress,
		>();
		let map_asset_progress =
			TLoading::register_load_tracking::<MapAsset<TCell>, LoadingGame, AssetsProgress>();

		//save maps
		TSavegame::register_savable_component::<MapAsset<TCell>>(self);
		self.register_required_components::<MapAsset<TCell>, TSavegame::TSaveEntityMarker>();

		// Track wether assets have been loaded
		map_lookup_progress.in_app(self, resource_exists::<ColorLookup<TCell>>);
		map_asset_progress.in_app(self, MapImage::<TCell>::all_loaded);

		self
			// Map color lookup
			.add_systems(
				OnEnter(GameState::LoadingEssentialAssets),
				ColorLookupImage::<TCell>::lookup_images,
			)
			.add_systems(
				Update,
				ColorLookup::<TCell>::parse_images
					.pipe(OnError::log)
					.run_if(not(resource_exists::<ColorLookup<TCell>>)),
			)
			// Load map cells and root graph from image
			.add_observer(MapAsset::<TCell>::load_map_image)
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
