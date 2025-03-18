use crate::{
	grid_graph::{
		GridGraph,
		Obstacles,
		grid_context::{GridContext, GridDefinition, GridDefinitionError},
	},
	half_offset_grid::HalfOffsetGrid,
	map::Map,
	traits::{GridCellDistanceDefinition, SourcePath, grid_min::GridMin, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::{
	tools::handle::default_handle,
	traits::{load_asset::LoadAsset, thread_safe::ThreadSafe},
};
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct CurrentLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) map: Handle<Map<TCell>>,
	pub(crate) graph: Option<GridGraph>,
	pub(crate) half_offset_grid: Option<HalfOffsetGrid>,
}

impl<TCell> CurrentLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	const NO_MAP: Handle<Map<TCell>> = default_handle();

	pub(crate) fn load_asset(
		commands: Commands,
		map_loader: ResMut<AssetServer>,
		level: Option<Res<Self>>,
	) where
		TCell: SourcePath,
	{
		Self::load_asset_internal(commands, map_loader, level);
	}

	pub(crate) fn set_graph(
		mut level: ResMut<Self>,
		maps: Res<Assets<Map<TCell>>>,
	) -> Result<(), GridDefinitionError>
	where
		TCell: GridCellDistanceDefinition + IsWalkable + Clone,
	{
		if level.graph.is_some() {
			return Ok(());
		}
		let Some(cells) = level.get_map_cells(maps) else {
			return Ok(());
		};
		let Some((cell_count_x, cell_count_z)) = Self::get_cell_counts(&cells) else {
			return Ok(());
		};
		let grid_definition = GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance: TCell::CELL_DISTANCE,
		};
		let mut graph = GridGraph {
			nodes: HashMap::default(),
			extra: Obstacles::default(),
			context: GridContext::try_from(grid_definition)?,
		};

		let min = graph.context.grid_min();
		let mut position = min;

		for (z, cell_line) in cells.into_iter().enumerate() {
			for (x, cell) in cell_line.into_iter().enumerate() {
				let x = x as i32;
				let z = z as i32;
				graph.nodes.insert((x, z), position);
				position.x += TCell::CELL_DISTANCE;

				if !cell.is_walkable() {
					graph.extra.obstacles.insert((x, z));
				}
			}
			position.x = min.x;
			position.z += TCell::CELL_DISTANCE;
		}

		level.graph = Some(graph);
		Ok(())
	}

	fn load_asset_internal<TLoadMap>(
		mut commands: Commands,
		mut map_loader: ResMut<TLoadMap>,
		level: Option<Res<Self>>,
	) where
		TCell: SourcePath,
		TLoadMap: LoadAsset + Resource,
	{
		if matches!(level, Some(level) if level.map != Self::NO_MAP) {
			return;
		}

		commands.insert_resource(Self {
			map: map_loader.load_asset(TCell::source_path()),
			..default()
		});
	}

	fn get_map_cells(&self, maps: Res<Assets<Map<TCell>>>) -> Option<Vec<Vec<TCell>>>
	where
		TCell: GridCellDistanceDefinition + Clone,
	{
		let map_handle = &self.map;
		let map = maps.get(map_handle)?;

		Some(map.0.clone())
	}

	fn get_cell_counts(cells: &[Vec<TCell>]) -> Option<(usize, usize)> {
		let count_x = cells.iter().map(|line| line.len()).max()?;
		let count_z = cells.len();

		Some((count_x, count_z))
	}
}

impl<TCell> Default for CurrentLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn default() -> Self {
		Self {
			map: default(),
			graph: default(),
			half_offset_grid: default(),
		}
	}
}

#[cfg(test)]
mod test_load {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{
			load_asset::{LoadAsset, Path},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Cell;

	impl SourcePath for _Cell {
		fn source_path() -> Path {
			Path::from("aaa/bbb/ccc.file_format")
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LoadMap {
		mock: Mock_LoadMap,
	}

	#[automock]
	impl LoadAsset for _LoadMap {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(load_map);
		app.add_systems(
			Update,
			CurrentLevel::<_Cell>::load_asset_internal::<_LoadMap>,
		);

		app
	}

	#[test]
	fn insert_level() {
		let map = new_handle();
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(map.clone());
		}));

		app.update();

		assert_eq!(
			Some(&CurrentLevel { map, ..default() }),
			app.world().get_resource::<CurrentLevel<_Cell>>()
		);
	}

	#[test]
	fn do_nothing_if_level_already_loaded() {
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset::<Map<_Cell>, Path>()
				.never()
				.return_const(new_handle());
		}));
		app.insert_resource(CurrentLevel::<_Cell> {
			map: new_handle(),
			..default()
		});

		app.update();
	}

	#[test]
	fn update_level_if_map_is_default() {
		let map = new_handle();
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(map.clone());
		}));
		app.insert_resource(CurrentLevel::<_Cell> {
			map: default(),
			..default()
		});

		app.update();

		assert_eq!(
			Some(&CurrentLevel { map, ..default() }),
			app.world().get_resource::<CurrentLevel<_Cell>>()
		);
	}
}

#[cfg(test)]
mod test_set_graph {
	use super::*;
	use crate::{
		grid_graph::{
			Obstacles,
			grid_context::{GridContext, GridDefinition},
		},
		traits::{GridCellDistanceDefinition, is_walkable::IsWalkable},
	};
	use common::test_tools::utils::{SingleThreadedApp, new_handle};
	use std::collections::{HashMap, HashSet};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell {
		is_walkable: bool,
	}

	impl _Cell {
		fn walkable() -> Self {
			Self { is_walkable: true }
		}

		fn not_walkable() -> Self {
			Self { is_walkable: false }
		}
	}

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: f32 = 4.;
	}

	impl IsWalkable for _Cell {
		fn is_walkable(&self) -> bool {
			self.is_walkable
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), GridDefinitionError>);

	fn setup(cells: Vec<Vec<_Cell>>) -> App {
		let map = new_handle::<Map<_Cell>>();
		let mut app = App::new().single_threaded(Update);
		let mut maps = Assets::default();

		maps.insert(&map.clone(), Map(cells));
		app.insert_resource(maps);
		app.insert_resource(CurrentLevel { map, ..default() });
		app.add_systems(
			Update,
			CurrentLevel::<_Cell>::set_graph.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			}),
		);

		app
	}

	fn get_context<TCell>(cell_count_x: usize, cell_count_z: usize) -> GridContext
	where
		TCell: GridCellDistanceDefinition,
	{
		let grid_definition = GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance: TCell::CELL_DISTANCE,
		};
		GridContext::try_from(grid_definition).expect("FAULTY")
	}

	#[test]
	fn one_by_one_with_no_obstacles() {
		let mut app = setup(vec![vec![_Cell::walkable()]]);

		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([((0, 0), Vec3::new(0., 0., 0.))]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(1, 1),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn one_by_two_with_no_obstacles() {
		let mut app = setup(vec![vec![_Cell::walkable()], vec![_Cell::walkable()]]);

		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(0., 0., -2.)),
					((0, 1), Vec3::new(0., 0., 2.))
				]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(1, 2),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn center_map() {
		let mut app = setup(vec![
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
		]);

		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-4., 0., -4.)),
					((1, 0), Vec3::new(0., 0., -4.)),
					((2, 0), Vec3::new(4., 0., -4.)),
					((0, 1), Vec3::new(-4., 0., 0.)),
					((1, 1), Vec3::new(0., 0., 0.)),
					((2, 1), Vec3::new(4., 0., 0.)),
					((0, 2), Vec3::new(-4., 0., 4.)),
					((1, 2), Vec3::new(0., 0., 4.)),
					((2, 2), Vec3::new(4., 0., 4.)),
				]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(3, 3),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn center_map_with_different_row_lengths() {
		let mut app = setup(vec![
			vec![_Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable()],
		]);

		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-4., 0., -4.)),
					((1, 0), Vec3::new(0., 0., -4.)),
					((0, 1), Vec3::new(-4., 0., 0.)),
					((1, 1), Vec3::new(0., 0., 0.)),
					((2, 1), Vec3::new(4., 0., 0.)),
					((0, 2), Vec3::new(-4., 0., 4.)),
				]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(3, 3),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn set_obstacles() {
		let mut app = setup(vec![
			vec![_Cell::walkable(), _Cell::not_walkable()],
			vec![_Cell::not_walkable(), _Cell::not_walkable()],
		]);

		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-2., 0., -2.)),
					((0, 1), Vec3::new(-2., 0., 2.)),
					((1, 0), Vec3::new(2., 0., -2.)),
					((1, 1), Vec3::new(2., 0., 2.)),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([(0, 1), (1, 0), (1, 1)])
				},
				context: get_context::<_Cell>(2, 2),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn do_nothing_if_graph_is_already_set() {
		let mut app = setup(vec![vec![_Cell::walkable()]]);

		app.world_mut().resource_mut::<CurrentLevel<_Cell>>().graph = Some(GridGraph {
			nodes: HashMap::default(),
			extra: Obstacles::default(),
			context: get_context::<_Cell>(10, 20),
		});
		app.update();

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::default(),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(10, 20),
			}),
			app.world().resource::<CurrentLevel<_Cell>>().graph
		);
	}

	#[test]
	fn return_grid_error() {
		let mut app = setup(vec![vec![]]);

		app.update();

		assert_eq!(
			&_Result(Err(GridDefinitionError::CellCountZero)),
			app.world().resource::<_Result>()
		);
	}
}
