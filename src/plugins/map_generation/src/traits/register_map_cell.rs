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
		map::{MapAssetCells, MapAssetPath, MapGridGraph},
	},
	map_cells::MapCells,
	map_loader::TextLoader,
};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingGame,
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
		TCell: TypePath
			+ From<Option<char>>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents;
}

impl RegisterMapCell for App {
	fn register_map_cell<TLoading, TSavegame, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		TSavegame: ThreadSafe + HandlesSaving,
		TCell: TypePath
			+ From<Option<char>>
			+ Clone
			+ ThreadSafe
			+ GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents,
	{
		let resolving_dependencies =
			TLoading::processing_state::<LoadingGame, DependenciesProgress>();

		//save maps
		TSavegame::register_savable_component::<MapAssetPath<TCell>>(self);
		self.register_required_components::<MapAssetPath<TCell>, TSavegame::TSaveEntityMarker>();

		// Track wether assets have been loaded
		TLoading::register_load_tracking::<MapAssetCells<TCell>, LoadingGame, AssetsProgress>()
			.in_app(self, MapAssetCells::<TCell>::all_loaded);

		self
			// register cell asset
			.init_asset::<MapCells<TCell>>()
			.register_asset_loader(TextLoader::<MapCells<TCell>>::default())
			// Generate Cells and Graph from asset path
			.add_observer(MapAssetPath::<TCell>::insert_map_cells)
			.add_systems(
				OnEnter(resolving_dependencies),
				MapAssetCells::<TCell>::insert_map_graph.pipe(OnError::log),
			)
			// Generate grid for navigation
			.add_observer(MapGridGraph::<TCell>::spawn_child::<Grid>)
			.add_observer(
				Grid::compute_cells::<TCell>
					.pipe(Grid::spawn_cells)
					.pipe(OnError::log),
			)
			// Generate grid with 1/2 offset for map models
			.add_observer(MapGridGraph::<TCell>::spawn_child::<HalfOffsetGrid>)
			.add_observer(
				HalfOffsetGrid::compute_cells::<TCell>
					.pipe(HalfOffsetGrid::spawn_cells)
					.pipe(OnError::log),
			)
	}
}
