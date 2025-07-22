use crate::{
	components::map::{cells::MapCells, grid_graph::MapGridGraph},
	grid_graph::{
		GridGraph,
		Obstacles,
		grid_context::{CellCountZero, GridContext, GridDefinition},
	},
	traits::{GridCellDistanceDefinition, grid_min::GridMin, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::traits::{or_ok::OrOk, thread_safe::ThreadSafe, try_insert_on::TryInsertOn};
use std::collections::HashMap;

impl<TCell> MapCells<TCell>
where
	TCell: TypePath + ThreadSafe + GridCellDistanceDefinition + IsWalkable,
{
	pub(crate) fn insert_map_grid_graph(
		maps: Query<(Entity, &Self)>,
		mut commands: Commands,
	) -> Result<(), Vec<CellCountZero>> {
		let insert_map_graph = |(entity, map): (Entity, &Self)| {
			let grid_definition = GridDefinition {
				cell_count_x: map.size.x,
				cell_count_z: map.size.z,
				cell_distance: TCell::CELL_DISTANCE,
			};
			let context = match GridContext::try_from(grid_definition) {
				Ok(ctx) => ctx,
				Err(error) => return Some(error),
			};
			let mut graph = GridGraph {
				nodes: HashMap::default(),
				extra: Obstacles::default(),
				context,
			};
			let min = graph.context.grid_min();
			let mut position = min;

			for x in 0..map.size.x {
				for z in 0..map.size.z {
					graph.nodes.insert((x, z), position);
					position.z += *TCell::CELL_DISTANCE;

					if !is_walkable(map, x, z) {
						graph.extra.obstacles.insert((x, z));
					}
				}

				position.z = min.z;
				position.x += *TCell::CELL_DISTANCE;
			}

			commands.try_insert_on(entity, MapGridGraph::<TCell>::from(graph));
			None
		};

		maps.into_iter()
			.filter_map(insert_map_graph)
			.collect::<Vec<_>>()
			.or_ok(|| ())
	}
}

fn is_walkable<TCell>(map: &MapCells<TCell>, x: usize, z: usize) -> bool
where
	TCell: IsWalkable,
{
	map.cells
		.get(&(x, z))
		.map(|cell| cell.is_walkable())
		.unwrap_or(false)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::map::cells::Size,
		grid_graph::{
			GridGraph,
			Obstacles,
			grid_context::{CellCountZero, CellDistance, GridContext, GridDefinition},
		},
		traits::{GridCellDistanceDefinition, is_walkable::IsWalkable},
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::new_valid;
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

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
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 4.);
	}

	impl IsWalkable for _Cell {
		fn is_walkable(&self) -> bool {
			self.is_walkable
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
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
	fn one_by_one_with_no_obstacles() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: Size { x: 1, z: 1 },
				cells: HashMap::from([((0, 0), _Cell::walkable())]),
				..default()
			})
			.id();

		_ = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([((0, 0), Vec3::new(0., 0., 0.))]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(1, 1),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
		Ok(())
	}

	#[test]
	fn one_by_two_with_no_obstacles() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: Size { x: 1, z: 2 },
				cells: HashMap::from([((0, 0), _Cell::walkable()), ((0, 1), _Cell::walkable())]),
				..default()
			})
			.id();

		_ = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(0., 0., -2.)),
					((0, 1), Vec3::new(0., 0., 2.))
				]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(1, 2),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
		Ok(())
	}

	#[test]
	fn center_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: Size { x: 3, z: 3 },
				cells: HashMap::from([
					((0, 0), _Cell::walkable()),
					((0, 1), _Cell::walkable()),
					((0, 2), _Cell::walkable()),
					((1, 0), _Cell::walkable()),
					((1, 1), _Cell::walkable()),
					((1, 2), _Cell::walkable()),
					((2, 0), _Cell::walkable()),
					((2, 1), _Cell::walkable()),
					((2, 2), _Cell::walkable()),
				]),
				..default()
			})
			.id();

		_ = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-4., 0., -4.)),
					((0, 1), Vec3::new(-4., 0., 0.)),
					((0, 2), Vec3::new(-4., 0., 4.)),
					((1, 0), Vec3::new(0., 0., -4.)),
					((1, 1), Vec3::new(0., 0., 0.)),
					((1, 2), Vec3::new(0., 0., 4.)),
					((2, 0), Vec3::new(4., 0., -4.)),
					((2, 1), Vec3::new(4., 0., 0.)),
					((2, 2), Vec3::new(4., 0., 4.)),
				]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(3, 3),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);

		Ok(())
	}

	#[test]
	fn set_obstacles() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: Size { x: 2, z: 2 },
				cells: HashMap::from([
					((0, 0), _Cell::walkable()),
					((0, 1), _Cell::not_walkable()),
					((1, 0), _Cell::not_walkable()),
					((1, 1), _Cell::not_walkable()),
				]),
				..default()
			})
			.id();

		_ = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
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
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
		Ok(())
	}

	#[test]
	fn map_with_holes() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: Size { x: 3, z: 3 },
				cells: HashMap::from([
					((0, 0), _Cell::walkable()),
					((0, 2), _Cell::walkable()),
					((1, 0), _Cell::walkable()),
					((1, 1), _Cell::walkable()),
					((1, 2), _Cell::walkable()),
					((2, 2), _Cell::walkable()),
				]),
				..default()
			})
			.id();

		_ = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-4., 0., -4.)),
					((0, 1), Vec3::new(-4., 0., 0.)),
					((0, 2), Vec3::new(-4., 0., 4.)),
					((1, 0), Vec3::new(0., 0., -4.)),
					((1, 1), Vec3::new(0., 0., 0.)),
					((1, 2), Vec3::new(0., 0., 4.)),
					((2, 0), Vec3::new(4., 0., -4.)),
					((2, 1), Vec3::new(4., 0., 0.)),
					((2, 2), Vec3::new(4., 0., 4.)),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([(0, 1), (2, 0), (2, 1)])
				},
				context: get_context::<_Cell>(3, 3),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
		Ok(())
	}

	#[test]
	fn return_grid_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(MapCells::<_Cell> {
			size: Size { x: 0, z: 0 },
			cells: HashMap::from([]),
			..default()
		});

		let result = app
			.world_mut()
			.run_system_once(MapCells::<_Cell>::insert_map_grid_graph)?;

		assert_eq!(Err(vec![CellCountZero]), result);
		Ok(())
	}
}
