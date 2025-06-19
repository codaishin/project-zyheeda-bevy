use crate::{
	grid_graph::{
		GridGraph,
		Obstacles,
		grid_context::{GridContext, GridDefinition, GridDefinitionError},
	},
	map_cells::{MapCells, half_offset_cell::HalfOffsetCell},
	traits::{GridCellDistanceDefinition, grid_min::GridMin, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level as ErrorLevel},
	traits::{thread_safe::ThreadSafe, try_despawn::TryDespawn},
};
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct Level<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) map: Handle<MapCells<TCell>>,
	pub(crate) graph: Option<GridGraph>,
}

impl<TCell> Level<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn set_graph(
		mut level: ResMut<Self>,
		maps: Res<Assets<MapCells<TCell>>>,
	) -> Result<(), SetGraphError>
	where
		TCell: GridCellDistanceDefinition + IsWalkable + Clone,
	{
		if level.graph.is_some() {
			return Err(SetGraphError::GraphAlreadySet);
		}
		let Some(cells) = level.get_map_cells(&maps) else {
			return Err(SetGraphError::MapAssetNotFound);
		};
		let (cell_count_x, cell_count_z) = Self::get_cell_counts(cells);
		let grid_definition = GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance: TCell::CELL_DISTANCE,
		};
		let mut graph = GridGraph {
			nodes: HashMap::default(),
			extra: Obstacles::default(),
			context: GridContext::try_from(grid_definition)
				.map_err(SetGraphError::GridDefinitionError)?,
		};

		let min = graph.context.grid_min();
		let mut position = min;

		for (z, cell_line) in cells.iter().enumerate() {
			for (x, cell) in cell_line.iter().enumerate() {
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

	pub(crate) fn grid_cells(
		level: Res<Self>,
		maps: Res<Assets<MapCells<TCell>>>,
	) -> Result<Vec<(Vec3, TCell)>, GridError>
	where
		TCell: Clone,
	{
		let Some(graph) = level.graph.as_ref() else {
			return Err(GridError::NoGridGraph);
		};
		let Some(map) = maps.get(&level.map) else {
			return Err(GridError::NoValidMap);
		};

		let cells = map.cells();
		let mut index_mismatch = None;
		let cell_translation = |((x, z), translation): (&(usize, usize), &Vec3)| {
			let x = *x;
			let z = *z;
			let Some(cell) = cells.get(z).and_then(|cells| cells.get(x)) else {
				index_mismatch = Some((x, z));
				return None;
			};

			Some((*translation, cell.clone()))
		};

		let grid = graph.nodes.iter().filter_map(cell_translation).collect();

		if let Some((x, z)) = index_mismatch {
			return Err(GridError::GridIndexHasNoCell { x, z });
		}

		Ok(grid)
	}

	pub(crate) fn half_offset_grid_cells(
		level: Res<Self>,
		maps: Res<Assets<MapCells<TCell>>>,
	) -> Result<Vec<(Vec3, HalfOffsetCell<TCell>)>, GridError>
	where
		TCell: Clone + GridCellDistanceDefinition,
	{
		let Some(graph) = level.graph.as_ref() else {
			return Err(GridError::NoGridGraph);
		};
		let Some(map) = maps.get(&level.map) else {
			return Err(GridError::NoValidMap);
		};

		let cells = map.half_offset_cells();
		let mut index_mismatch = None;
		let max_x = graph
			.nodes
			.keys()
			.map(|(x, _)| *x)
			.max()
			.unwrap_or_default();
		let max_z = graph
			.nodes
			.keys()
			.map(|(_, z)| *z)
			.max()
			.unwrap_or_default();
		let cell_translation = |((x, z), translation): (&(usize, usize), &Vec3)| {
			let x = *x;
			let z = *z;
			if x == max_x || z == max_z {
				return None;
			}
			let Some(cell) = cells.get(z).and_then(|cells| cells.get(x)) else {
				index_mismatch = Some((x, z));
				return None;
			};
			let offset = TCell::CELL_DISTANCE / 2.;
			let translation = *translation + Vec3::new(offset, 0., offset);

			Some((translation, cell.clone()))
		};

		let grid = graph.nodes.iter().filter_map(cell_translation).collect();

		if let Some((x, z)) = index_mismatch {
			return Err(GridError::GridIndexHasNoCell { x, z });
		}

		Ok(grid)
	}

	pub(crate) fn spawn_unique<TGrid>(
		level: Res<Self>,
		mut commands: Commands,
		grids: Query<Entity, With<TGrid>>,
	) -> Result<(), NoGridGraphSet>
	where
		TGrid: Component + From<GridGraph>,
	{
		let Some(graph) = &level.graph else {
			return Err(NoGridGraphSet);
		};

		for grid in &grids {
			commands.try_despawn(grid);
		}

		commands.spawn((TGrid::from(graph.clone()), Transform::default()));
		Ok(())
	}

	fn get_map_cells<'a>(&self, maps: &'a Assets<MapCells<TCell>>) -> Option<&'a Vec<Vec<TCell>>>
	where
		TCell: GridCellDistanceDefinition + Clone,
	{
		maps.get(&self.map).map(|m| m.cells())
	}

	fn get_cell_counts(cells: &[Vec<TCell>]) -> (usize, usize) {
		let count_x = cells
			.iter()
			.map(|line| line.len())
			.max()
			.unwrap_or_default();
		let count_z = cells.len();

		(count_x, count_z)
	}
}

impl<TCell> Default for Level<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn default() -> Self {
		Self {
			map: default(),
			graph: default(),
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum SetGraphError {
	GridDefinitionError(GridDefinitionError),
	GraphAlreadySet,
	MapAssetNotFound,
}

impl From<SetGraphError> for Error {
	fn from(error: SetGraphError) -> Self {
		match error {
			SetGraphError::GridDefinitionError(error) => Error::from(error),
			SetGraphError::GraphAlreadySet => Error {
				msg: "Grid graph was already set".to_owned(),
				lvl: ErrorLevel::Warning,
			},
			SetGraphError::MapAssetNotFound => Error {
				msg: "Map asset not found".to_owned(),
				lvl: ErrorLevel::Error,
			},
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum GridError {
	NoValidMap,
	NoGridGraph,
	GridIndexHasNoCell { x: usize, z: usize },
}

impl From<GridError> for Error {
	fn from(error: GridError) -> Self {
		Self {
			msg: format!("Faulty grid encountered: {:?}", error),
			lvl: ErrorLevel::Error,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoGridGraphSet;

impl From<NoGridGraphSet> for Error {
	fn from(_: NoGridGraphSet) -> Self {
		Self {
			msg: "Grid graph was not set".to_owned(),
			lvl: ErrorLevel::Error,
		}
	}
}

#[cfg(test)]
mod test_spawn {
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
	struct _Result(Result<(), SetGraphError>);

	fn setup(cells: Vec<Vec<_Cell>>) -> App {
		let map = new_handle::<MapCells<_Cell>>();
		let mut app = App::new().single_threaded(Update);
		let mut maps = Assets::default();

		maps.insert(&map.clone(), MapCells::new(cells, vec![]));
		app.insert_resource(maps);
		app.insert_resource(Level { map, ..default() });
		app.add_systems(
			Update,
			Level::<_Cell>::set_graph.pipe(|In(result), mut commands: Commands| {
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
			app.world().resource::<Level<_Cell>>().graph
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
			app.world().resource::<Level<_Cell>>().graph
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
			app.world().resource::<Level<_Cell>>().graph
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
			app.world().resource::<Level<_Cell>>().graph
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
			app.world().resource::<Level<_Cell>>().graph
		);
	}

	#[test]
	fn do_nothing_if_graph_is_already_set() {
		let mut app = setup(vec![vec![_Cell::walkable()]]);

		app.world_mut().resource_mut::<Level<_Cell>>().graph = Some(GridGraph {
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
			app.world().resource::<Level<_Cell>>().graph
		);
	}

	#[test]
	fn return_grid_error() {
		let mut app = setup(vec![vec![]]);

		app.update();

		assert_eq!(
			&_Result(Err(SetGraphError::GridDefinitionError(
				GridDefinitionError::CellCountZero
			))),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_graph_already_set_error() {
		let mut app = setup(vec![vec![]]);
		app.world_mut().resource_mut::<Level<_Cell>>().graph = Some(GridGraph::default());

		app.update();

		assert_eq!(
			&_Result(Err(SetGraphError::GraphAlreadySet)),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_asset_error() {
		let mut app = setup(vec![vec![]]);
		let mut assets = app.world_mut().resource_mut::<Assets<MapCells<_Cell>>>();
		*assets = Assets::default();

		app.update();

		assert_eq!(
			&_Result(Err(SetGraphError::MapAssetNotFound)),
			app.world().resource::<_Result>()
		);
	}
}

#[cfg(test)]
mod test_get_grid {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		assert_eq_unordered,
		test_tools::utils::{SingleThreadedApp, new_handle},
	};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	fn setup(cells: Option<Vec<Vec<_Cell>>>, graph: Option<GridGraph>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut maps = Assets::default();

		let map = if let Some(cells) = cells {
			let map = new_handle::<MapCells<_Cell>>();
			maps.insert(&map.clone(), MapCells::new(cells, vec![]));
			map
		} else {
			Handle::default()
		};

		app.insert_resource(maps);
		app.insert_resource(Level { map, graph });

		app
	}

	#[test]
	fn return_grid_cells_with_translation() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let cells = vec![
			vec![_Cell("00"), _Cell("10")],
			vec![_Cell("01"), _Cell("11")],
		];
		let mut app = setup(Some(cells), Some(graph));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::grid_cells)?;

		assert_eq_unordered!(
			Ok(vec![
				(Vec3::new(-1., 0., -1.), _Cell("00")),
				(Vec3::new(-1., 0., 1.), _Cell("01")),
				(Vec3::new(1., 0., -1.), _Cell("10")),
				(Vec3::new(1., 0., 1.), _Cell("11")),
			]),
			grid
		);
		Ok(())
	}

	#[test]
	fn grid_error_no_valid_map() -> Result<(), RunSystemError> {
		let mut app = setup(None, Some(GridGraph::default()));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::grid_cells)?;

		assert_eq_unordered!(Err(GridError::NoValidMap), grid);
		Ok(())
	}

	#[test]
	fn grid_error_no_grid_graph() -> Result<(), RunSystemError> {
		let mut app = setup(Some(vec![]), None);

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::grid_cells)?;

		assert_eq_unordered!(Err(GridError::NoGridGraph), grid);
		Ok(())
	}

	#[test]
	fn grid_error_graph_index_has_no_cell() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let cells = vec![vec![_Cell("00")], vec![_Cell("01"), _Cell("11")]];
		let mut app = setup(Some(cells), Some(graph));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::grid_cells)?;

		assert_eq_unordered!(Err(GridError::GridIndexHasNoCell { x: 1, z: 0 }), grid);
		Ok(())
	}
}

#[cfg(test)]
mod test_get_half_offset_grid {
	use super::*;
	use crate::map_cells::Direction;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		assert_eq_unordered,
		test_tools::utils::{SingleThreadedApp, new_handle},
	};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: f32 = 2.;
	}

	fn setup(quadrants: Option<Vec<Vec<HalfOffsetCell<_Cell>>>>, graph: Option<GridGraph>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut maps = Assets::default();

		let map = if let Some(quadrants) = quadrants {
			let map = new_handle::<MapCells<_Cell>>();
			maps.insert(&map.clone(), MapCells::new(vec![], quadrants));
			map
		} else {
			Handle::default()
		};

		app.insert_resource(maps);
		app.insert_resource(Level { map, graph });

		app
	}

	#[test]
	fn return_grid_cells_with_translation() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let cells = vec![vec![HalfOffsetCell::from([
			(Direction::Z, _Cell("00")),
			(Direction::NegX, _Cell("01")),
			(Direction::NegZ, _Cell("11")),
			(Direction::X, _Cell("10")),
		])]];
		let mut app = setup(Some(cells), Some(graph));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::half_offset_grid_cells)?;

		assert_eq_unordered!(
			Ok(vec![(
				Vec3::new(0., 0., 0.),
				HalfOffsetCell::from([
					(Direction::Z, _Cell("00")),
					(Direction::X, _Cell("10")),
					(Direction::NegX, _Cell("01")),
					(Direction::NegZ, _Cell("11"))
				])
			),]),
			grid
		);
		Ok(())
	}

	#[test]
	fn grid_error_no_valid_map() -> Result<(), RunSystemError> {
		let mut app = setup(None, Some(GridGraph::default()));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::half_offset_grid_cells)?;

		assert_eq_unordered!(Err(GridError::NoValidMap), grid);
		Ok(())
	}

	#[test]
	fn grid_error_no_grid_graph() -> Result<(), RunSystemError> {
		let mut app = setup(Some(vec![]), None);

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::half_offset_grid_cells)?;

		assert_eq_unordered!(Err(GridError::NoGridGraph), grid);
		Ok(())
	}

	#[test]
	fn grid_error_graph_index_has_no_cell() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
				((2, 1), Vec3::new(2., -0., 2.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let cells = vec![vec![HalfOffsetCell::from([
			(Direction::Z, _Cell("00")),
			(Direction::NegX, _Cell("01")),
			(Direction::NegZ, _Cell("11")),
			(Direction::X, _Cell("10")),
		])]];
		let mut app = setup(Some(cells), Some(graph));

		let grid = app
			.world_mut()
			.run_system_once(Level::<_Cell>::half_offset_grid_cells)?;

		assert_eq_unordered!(Err(GridError::GridIndexHasNoCell { x: 1, z: 0 }), grid);
		Ok(())
	}
}

#[cfg(test)]
mod test_spawn_unique {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{assert_count, get_children, test_tools::utils::SingleThreadedApp};

	#[derive(TypePath)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Grid(GridGraph);

	impl From<GridGraph> for _Grid {
		fn from(graph: GridGraph) -> Self {
			Self(graph)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_grid() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let mut app = setup();
		app.insert_resource(Level {
			map: Handle::<MapCells<_Cell>>::default(),
			graph: Some(graph.clone()),
		});

		let result = app
			.world_mut()
			.run_system_once(Level::<_Cell>::spawn_unique::<_Grid>)?;

		let [grid] = assert_count!(1, app.world().iter_entities());
		assert_eq!(
			Some(&_Grid(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-1., -0., -1.)),
					((0, 1), Vec3::new(-1., -0., 1.)),
					((1, 0), Vec3::new(1., -0., -1.)),
					((1, 1), Vec3::new(1., -0., 1.)),
				]),
				extra: Obstacles::default(),
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 2,
					cell_count_z: 2,
					cell_distance: 2.,
				})
				.expect("INVALID GRID DEFINITION"),
			})),
			grid.get::<_Grid>()
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn spawn_grid_with_transform() -> Result<(), RunSystemError> {
		let graph = GridGraph::default();
		let mut app = setup();
		app.insert_resource(Level {
			map: Handle::<MapCells<_Cell>>::default(),
			graph: Some(graph.clone()),
		});

		let result = app
			.world_mut()
			.run_system_once(Level::<_Cell>::spawn_unique::<_Grid>)?;

		let [grid] = assert_count!(1, app.world().iter_entities());
		assert_eq!(Some(&Transform::default()), grid.get::<Transform>());
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn despawn_all_other_grids() -> Result<(), RunSystemError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-1., -0., -1.)),
				((0, 1), Vec3::new(-1., -0., 1.)),
				((1, 0), Vec3::new(1., -0., -1.)),
				((1, 1), Vec3::new(1., -0., 1.)),
			]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let mut app = setup();
		app.world_mut()
			.spawn(_Grid(GridGraph::default()))
			.with_child(());
		app.insert_resource(Level {
			map: Handle::<MapCells<_Cell>>::default(),
			graph: Some(graph.clone()),
		});

		let result = app
			.world_mut()
			.run_system_once(Level::<_Cell>::spawn_unique::<_Grid>)?;

		let [grid] = assert_count!(1, app.world().iter_entities());
		assert_eq!(
			Some(&_Grid(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-1., -0., -1.)),
					((0, 1), Vec3::new(-1., -0., 1.)),
					((1, 0), Vec3::new(1., -0., -1.)),
					((1, 1), Vec3::new(1., -0., 1.)),
				]),
				extra: Obstacles::default(),
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 2,
					cell_count_z: 2,
					cell_distance: 2.,
				})
				.expect("INVALID GRID DEFINITION"),
			})),
			grid.get::<_Grid>()
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn do_not_despawn_all_other_non_grids() -> Result<(), RunSystemError> {
		#[derive(Component, Debug, PartialEq)]
		struct _NotAGrid;

		let graph = GridGraph::default();
		let mut app = setup();
		let entity = app.world_mut().spawn(_NotAGrid).with_child(_NotAGrid).id();
		app.insert_resource(Level {
			map: Handle::<MapCells<_Cell>>::default(),
			graph: Some(graph.clone()),
		});

		let result = app
			.world_mut()
			.run_system_once(Level::<_Cell>::spawn_unique::<_Grid>)?;

		let [child] = assert_count!(1, get_children!(app, entity));
		let parent = app.world().entity(entity);
		assert_eq!(
			[true, true],
			[
				parent.contains::<_NotAGrid>(),
				child.contains::<_NotAGrid>()
			]
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn return_error_when_grid_graph_is_none() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.insert_resource(Level {
			map: Handle::<MapCells<_Cell>>::default(),
			graph: None,
		});

		let result = app
			.world_mut()
			.run_system_once(Level::<_Cell>::spawn_unique::<_Grid>)?;

		assert_eq!(Err(NoGridGraphSet), result);
		Ok(())
	}
}
