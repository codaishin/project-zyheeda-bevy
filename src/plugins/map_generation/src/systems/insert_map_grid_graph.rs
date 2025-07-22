use crate::{
	components::map::{cells::MapCells, grid_graph::MapGridGraph},
	grid_graph::{GridGraph, Obstacles, grid_context::GridContext},
	traits::{GridCellDistanceDefinition, grid_min::GridMin, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};
use std::collections::HashMap;

impl<TCell> MapCells<TCell>
where
	TCell: TypePath + ThreadSafe + GridCellDistanceDefinition + IsWalkable,
{
	pub(crate) fn insert_map_grid_graph(maps: Query<(Entity, &Self)>, mut commands: Commands) {
		for (entity, map) in &maps {
			let context = GridContext {
				cell_count_x: map.size.x,
				cell_count_z: map.size.z,
				cell_distance: TCell::CELL_DISTANCE,
			};
			let mut graph = GridGraph {
				nodes: HashMap::default(),
				extra: Obstacles::default(),
				context,
			};
			let min = graph.context.grid_min();
			let mut position = min;

			for x in 0..*map.size.x {
				for z in 0..*map.size.z {
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
		}
	}
}

fn is_walkable<TCell>(map: &MapCells<TCell>, x: u32, z: u32) -> bool
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
		cell_grid_size::CellGridSize,
		grid_graph::{
			GridGraph,
			Obstacles,
			grid_context::{CellCount, CellDistance, GridContext},
		},
		traits::{GridCellDistanceDefinition, is_walkable::IsWalkable},
	};
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
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, MapCells::<_Cell>::insert_map_grid_graph);

		app
	}

	#[test]
	fn two_by_two_with_no_obstacles() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 2),
					z: new_valid!(CellCount, 2),
				},
				cells: HashMap::from([
					((0, 0), _Cell::walkable()),
					((0, 1), _Cell::walkable()),
					((1, 0), _Cell::walkable()),
					((1, 1), _Cell::walkable()),
				]),
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-2., 0., -2.)),
					((0, 1), Vec3::new(-2., 0., 2.)),
					((1, 0), Vec3::new(2., 0., -2.)),
					((1, 1), Vec3::new(2., 0., 2.)),
				]),
				extra: Obstacles::default(),
				context: GridContext {
					cell_count_x: new_valid!(CellCount, 2),
					cell_count_z: new_valid!(CellCount, 2),
					cell_distance: _Cell::CELL_DISTANCE
				},
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn center_map() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 3),
					z: new_valid!(CellCount, 3),
				},
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

		app.update();

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
				context: GridContext {
					cell_count_x: new_valid!(CellCount, 3),
					cell_count_z: new_valid!(CellCount, 3),
					cell_distance: _Cell::CELL_DISTANCE
				},
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn set_obstacles() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 2),
					z: new_valid!(CellCount, 2),
				},
				cells: HashMap::from([
					((0, 0), _Cell::walkable()),
					((0, 1), _Cell::not_walkable()),
					((1, 0), _Cell::not_walkable()),
					((1, 1), _Cell::not_walkable()),
				]),
				..default()
			})
			.id();

		app.update();

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
				context: GridContext {
					cell_count_x: new_valid!(CellCount, 2),
					cell_count_z: new_valid!(CellCount, 2),
					cell_distance: _Cell::CELL_DISTANCE
				},
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn map_with_holes() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 3),
					z: new_valid!(CellCount, 3),
				},
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

		app.update();

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
				context: GridContext {
					cell_count_x: new_valid!(CellCount, 3),
					cell_count_z: new_valid!(CellCount, 3),
					cell_distance: _Cell::CELL_DISTANCE
				},
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}
}
