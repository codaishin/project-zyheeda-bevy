use crate::{
	grid_graph::{GridGraph, grid_context::GridDefinitionError},
	map_cells::{MapCells, half_offset_cell::HalfOffsetCell},
	traits::GridCellDistanceDefinition,
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level as ErrorLevel},
	traits::{thread_safe::ThreadSafe, try_despawn::TryDespawn},
};

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
	MapAssetNotFound,
}

impl From<SetGraphError> for Error {
	fn from(error: SetGraphError) -> Self {
		match error {
			SetGraphError::GridDefinitionError(error) => Error::from(error),
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
mod test_get_grid {
	use std::collections::HashMap;

	use crate::grid_graph::{
		Obstacles,
		grid_context::{GridContext, GridDefinition},
	};

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
	use std::collections::HashMap;

	use super::*;
	use crate::{
		grid_graph::{
			Obstacles,
			grid_context::{GridContext, GridDefinition},
		},
		map_cells::Direction,
	};
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
	use std::collections::HashMap;

	use crate::grid_graph::{
		Obstacles,
		grid_context::{GridContext, GridDefinition},
	};

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
