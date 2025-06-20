use crate::{
	components::{
		cells_ref::CellsRef,
		half_offset_grid::HalfOffsetGrid,
		map::{MapAssetCells, MapGridGraph},
	},
	errors::GridError,
	map_cells::{MapCells, half_offset_cell::HalfOffsetCell},
	traits::GridCellDistanceDefinition,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

pub(crate) type Cells<TCell> = (Entity, Vec<(Vec3, HalfOffsetCell<TCell>)>);

impl HalfOffsetGrid {
	pub(crate) fn compute_cells<TCell>(
		trigger: Trigger<OnAdd, Self>,
		grids: Query<&CellsRef<TCell>, With<Self>>,
		maps: Query<(&MapAssetCells<TCell>, &MapGridGraph<TCell>)>,
		assets: Res<Assets<MapCells<TCell>>>,
	) -> Result<Cells<TCell>, GridError>
	where
		TCell: TypePath + ThreadSafe + GridCellDistanceDefinition + Clone,
	{
		let entity = trigger.target();
		let Ok(cells_ref) = grids.get(entity) else {
			return Err(GridError::NoRefToCellDefinition);
		};
		let Ok((cells, graph)) = maps.get(cells_ref.cell_definition) else {
			return Err(GridError::NoCellDefinition);
		};
		let Some(map) = assets.get(cells.cells()) else {
			return Err(GridError::NoValidMap);
		};

		let cells = map.half_offset_cells();
		let graph = graph.graph();
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

		let cells = graph.nodes.iter().filter_map(cell_translation).collect();

		if let Some((x, z)) = index_mismatch {
			return Err(GridError::GridIndexHasNoCell { x, z });
		}

		Ok((entity, cells))
	}
}

#[cfg(test)]
mod test_get_half_offset_grid {
	use super::*;
	use crate::{
		components::{cells_ref::CellsRef, map::MapAssetCells},
		grid_graph::{
			GridGraph,
			Obstacles,
			grid_context::{GridContext, GridDefinition},
		},
		map_cells::{Direction, MapCells},
		traits::GridCellDistanceDefinition,
	};
	use common::{
		assert_eq_unordered,
		test_tools::utils::{SingleThreadedApp, new_handle},
	};
	use std::collections::HashMap;

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: f32 = 2.;
	}

	#[derive(Resource, Debug, PartialEq, Clone)]
	struct _Result(Result<Cells<_Cell>, GridError>);

	fn setup(
		quadrants: Option<Vec<Vec<HalfOffsetCell<_Cell>>>>,
		graph: GridGraph,
	) -> (App, Entity) {
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
		app.add_observer(HalfOffsetGrid::compute_cells::<_Cell>.pipe(
			|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			},
		));
		let cell_definition = app
			.world_mut()
			.spawn((MapAssetCells::from(map), MapGridGraph::<_Cell>::from(graph)))
			.id();

		(app, cell_definition)
	}

	#[test]
	fn return_grid_cells_with_translation() {
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
		let (mut app, cell_definition) = setup(Some(cells), graph);

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cell_definition),
		));

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
			app.world()
				.resource::<_Result>()
				.0
				.clone()
				.map(|(_, cells)| cells)
		);
	}

	#[test]
	fn return_grid_entity() {
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
		let (mut app, cell_definition) = setup(Some(cells), graph);

		let grid = app
			.world_mut()
			.spawn((
				HalfOffsetGrid,
				CellsRef::<_Cell>::from_grid_definition(cell_definition),
			))
			.id();

		assert_eq!(
			Ok(grid),
			app.world()
				.resource::<_Result>()
				.0
				.clone()
				.map(|(entity, _)| entity)
		);
	}

	#[test]
	fn grid_error_no_valid_map() {
		let (mut app, cell_definition) = setup(None, GridGraph::default());

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cell_definition),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::NoValidMap))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn grid_error_graph_index_has_no_cell() {
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
		let (mut app, cell_definition) = setup(Some(cells), graph);

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cell_definition),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::GridIndexHasNoCell { x: 1, z: 0 }))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_cell_ref() {
		let grid = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 1.,
			})
			.expect("INVALID GRID"),
		};
		let (mut app, _) = setup(Some(vec![]), grid);

		app.world_mut().spawn(HalfOffsetGrid);

		assert_eq!(
			Some(&_Result(Err(GridError::NoRefToCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_valid_cell_ref() {
		let grid = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 1.,
			})
			.expect("INVALID GRID"),
		};
		let (mut app, _) = setup(Some(vec![]), grid);

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(Entity::from_raw(123)),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::NoCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}
}
