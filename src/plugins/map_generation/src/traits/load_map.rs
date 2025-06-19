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
		map::{MapAssetCells, MapAssetPath},
	},
	map_cells::MapCells,
	map_loader::TextLoader,
	resources::level::Level,
};
use bevy::{
	app::App,
	asset::AssetApp,
	ecs::{
		schedule::{IntoScheduleConfigs, ScheduleLabel},
		system::IntoSystem,
	},
	reflect::TypePath,
};
use common::{
	states::game_state::LoadingGame,
	systems::log::log,
	traits::{
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		thread_safe::ThreadSafe,
	},
};

pub(crate) trait RegisterMapAsset {
	fn register_map_asset<TLoading, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		TCell: TypePath + From<Option<char>> + Clone + ThreadSafe;
}

pub(crate) trait LoadMap {
	fn load_map<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App
	where
		TCell: GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ TypePath
			+ Clone
			+ ThreadSafe;
}

impl RegisterMapAsset for App {
	fn register_map_asset<TLoading, TCell>(&mut self) -> &mut App
	where
		TLoading: ThreadSafe + HandlesLoadTracking,
		TCell: TypePath + From<Option<char>> + Clone + ThreadSafe,
	{
		TLoading::register_load_tracking::<MapAssetCells<TCell>, LoadingGame, AssetsProgress>()
			.in_app(self, MapAssetCells::<TCell>::all_loaded);

		self.init_asset::<MapCells<TCell>>()
			.register_asset_loader(TextLoader::<MapCells<TCell>>::default())
			.add_observer(MapAssetPath::<TCell>::insert_map_cells)
	}
}

impl LoadMap for App {
	fn load_map<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App
	where
		TCell: GridCellDistanceDefinition
			+ IsWalkable
			+ InsertCellComponents
			+ InsertCellQuadrantComponents
			+ TypePath
			+ Clone
			+ ThreadSafe,
	{
		self.add_systems(
			label,
			(
				Level::<TCell>::set_graph.pipe(log),
				Level::<TCell>::spawn_unique::<Grid>.pipe(log),
				Level::<TCell>::spawn_unique::<HalfOffsetGrid>.pipe(log),
				Level::<TCell>::grid_cells.pipe(Grid::spawn_cells).pipe(log),
				Level::<TCell>::half_offset_grid_cells
					.pipe(HalfOffsetGrid::spawn_cells)
					.pipe(log),
			)
				.chain(),
		)
	}
}
