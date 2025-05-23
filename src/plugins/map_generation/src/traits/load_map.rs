use super::{
	GridCellDistanceDefinition,
	SourcePath,
	insert_cell_components::InsertCellComponents,
	insert_cell_quadrant_components::InsertCellQuadrantComponents,
	is_walkable::IsWalkable,
};
use crate::{
	components::{grid::Grid, half_offset_grid::HalfOffsetGrid},
	map::Map,
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
use common::{systems::log::log, traits::thread_safe::ThreadSafe};

pub(crate) trait LoadMapAsset {
	fn load_map_asset<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App
	where
		TCell: TypePath + From<Option<char>> + SourcePath + Clone + ThreadSafe;
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

impl LoadMapAsset for App {
	fn load_map_asset<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App
	where
		TCell: TypePath + From<Option<char>> + SourcePath + Clone + ThreadSafe,
	{
		self.init_asset::<Map<TCell>>()
			.register_asset_loader(TextLoader::<Map<TCell>>::default())
			.add_systems(label, Level::<TCell>::load_asset)
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
