use crate::{
	components::map::{MapAssetCells, MapGridGraph},
	errors::InsertGraphError,
	grid_graph::{
		GridGraph,
		Obstacles,
		grid_context::{GridContext, GridDefinition},
	},
	map_cells::MapCells,
	traits::{GridCellDistanceDefinition, grid_min::GridMin, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};
use std::collections::HashMap;

impl<TCell> MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe + GridCellDistanceDefinition + IsWalkable,
{
	pub(crate) fn insert_map_graph(
		maps: Query<(Entity, &Self)>,
		assets: Res<Assets<MapCells<TCell>>>,
		mut commands: Commands,
	) -> Vec<Result<(), InsertGraphError>> {
		let insert_map_graph = |(entity, map): (Entity, &Self)| {
			let Some(asset) = assets.get(map.cells()) else {
				return Err(InsertGraphError::MapAssetNotFound);
			};
			let (cell_count_x, cell_count_z) = get_cell_counts(asset.cells());
			let grid_definition = GridDefinition {
				cell_count_x,
				cell_count_z,
				cell_distance: TCell::CELL_DISTANCE,
			};
			let mut graph = GridGraph {
				nodes: HashMap::default(),
				extra: Obstacles::default(),
				context: GridContext::try_from(grid_definition)
					.map_err(InsertGraphError::GridDefinitionError)?,
			};
			let min = graph.context.grid_min();
			let mut position = min;

			for (z, cell_line) in asset.cells().iter().enumerate() {
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

			commands.try_insert_on(entity, MapGridGraph::<TCell>::from(graph));
			Ok(())
		};

		maps.into_iter().map(insert_map_graph).collect()
	}
}

fn get_cell_counts<TCell>(cells: &[Vec<TCell>]) -> (usize, usize) {
	let count_x = cells
		.iter()
		.map(|line| line.len())
		.max()
		.unwrap_or_default();
	let count_z = cells.len();

	(count_x, count_z)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::map::MapGridGraph,
		grid_graph::{
			GridGraph,
			Obstacles,
			grid_context::{GridContext, GridDefinition, GridDefinitionError},
		},
		map_cells::MapCells,
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
	struct _Results(Vec<Result<(), InsertGraphError>>);

	fn setup(cells: Vec<Vec<_Cell>>) -> (App, Entity) {
		let cells_handle = new_handle();
		let mut app = App::new().single_threaded(Update);
		let mut maps = Assets::default();

		maps.insert(&cells_handle.clone(), MapCells::new(cells, vec![]));
		app.insert_resource(maps);
		let entity = app
			.world_mut()
			.spawn(MapAssetCells::from(cells_handle))
			.id();
		app.add_systems(
			Update,
			MapAssetCells::<_Cell>::insert_map_graph.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Results(result));
			}),
		);

		(app, entity)
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
		let (mut app, entity) = setup(vec![vec![_Cell::walkable()]]);

		app.update();

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
				nodes: HashMap::from([((0, 0), Vec3::new(0., 0., 0.))]),
				extra: Obstacles::default(),
				context: get_context::<_Cell>(1, 1),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn one_by_two_with_no_obstacles() {
		let (mut app, entity) = setup(vec![vec![_Cell::walkable()], vec![_Cell::walkable()]]);

		app.update();

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
	}

	#[test]
	fn center_map() {
		let (mut app, entity) = setup(vec![
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
		]);

		app.update();

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
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
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn center_map_with_different_row_lengths() {
		let (mut app, entity) = setup(vec![
			vec![_Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable(), _Cell::walkable(), _Cell::walkable()],
			vec![_Cell::walkable()],
		]);

		app.update();

		assert_eq!(
			Some(&MapGridGraph::from(GridGraph {
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
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn set_obstacles() {
		let (mut app, entity) = setup(vec![
			vec![_Cell::walkable(), _Cell::not_walkable()],
			vec![_Cell::not_walkable(), _Cell::not_walkable()],
		]);

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
				context: get_context::<_Cell>(2, 2),
			})),
			app.world().entity(entity).get::<MapGridGraph<_Cell>>()
		);
	}

	#[test]
	fn return_grid_error() {
		let (mut app, _) = setup(vec![vec![]]);

		app.update();

		assert_eq!(
			&_Results(vec![Err(InsertGraphError::GridDefinitionError(
				GridDefinitionError::CellCountZero
			))]),
			app.world().resource::<_Results>()
		);
	}

	#[test]
	fn return_asset_error() {
		let (mut app, _) = setup(vec![vec![]]);
		let mut assets = app.world_mut().resource_mut::<Assets<MapCells<_Cell>>>();
		*assets = Assets::default();

		app.update();

		assert_eq!(
			&_Results(vec![Err(InsertGraphError::MapAssetNotFound)]),
			app.world().resource::<_Results>()
		);
	}
}
