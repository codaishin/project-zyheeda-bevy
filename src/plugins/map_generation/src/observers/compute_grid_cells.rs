use crate::{
	components::{cells_ref::CellsRef, grid::Grid, map::MapAssetCells},
	errors::GridError,
	map_cells::MapCells,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

pub(crate) type Cells<TCell> = (Entity, Vec<(Vec3, TCell)>);

impl Grid {
	pub(crate) fn compute_cells<TCell>(
		trigger: Trigger<OnAdd, Self>,
		grids: Query<(&Self, &CellsRef<TCell>)>,
		cells: Query<&MapAssetCells<TCell>>,
		assets: Res<Assets<MapCells<TCell>>>,
	) -> Result<Cells<TCell>, GridError>
	where
		TCell: TypePath + ThreadSafe + Clone,
	{
		let entity = trigger.target();
		let Ok((grid, cells_ref)) = grids.get(entity) else {
			return Err(GridError::NoRefToCellDefinition);
		};
		let Ok(cells) = cells.get(cells_ref.cell_definition) else {
			return Err(GridError::NoCellDefinition);
		};
		let Some(map) = assets.get(cells.cells()) else {
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

		let cells = grid
			.graph()
			.nodes
			.iter()
			.filter_map(cell_translation)
			.collect();

		if let Some((x, z)) = index_mismatch {
			return Err(GridError::GridIndexHasNoCell { x, z });
		}

		Ok((entity, cells))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::map::MapAssetCells,
		grid_graph::{
			GridGraph,
			Obstacles,
			grid_context::{GridContext, GridDefinition},
		},
		map_cells::MapCells,
	};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_eq_unordered, new_handle};

	#[derive(Resource, Debug, PartialEq, Clone)]
	struct _Result(Result<(Entity, Vec<(Vec3, _Cell)>), GridError>);

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	fn setup(cells: Option<Vec<Vec<_Cell>>>) -> (App, Entity) {
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
		app.add_observer(
			Grid::compute_cells.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			}),
		);

		let cell_definition = app.world_mut().spawn(MapAssetCells::from(map)).id();

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
		let cells = vec![
			vec![_Cell("00"), _Cell("10")],
			vec![_Cell("01"), _Cell("11")],
		];
		let (mut app, cell_definition) = setup(Some(cells));

		app.world_mut().spawn((
			Grid::from(graph),
			CellsRef::<_Cell>::from_grid_definition(cell_definition),
		));

		assert_eq_unordered!(
			Ok(vec![
				(Vec3::new(-1., 0., -1.), _Cell("00")),
				(Vec3::new(-1., 0., 1.), _Cell("01")),
				(Vec3::new(1., 0., -1.), _Cell("10")),
				(Vec3::new(1., 0., 1.), _Cell("11")),
			]),
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
		let cells = vec![
			vec![_Cell("00"), _Cell("10")],
			vec![_Cell("01"), _Cell("11")],
		];
		let (mut app, cell_definition) = setup(Some(cells));

		let grid = app
			.world_mut()
			.spawn((
				Grid::from(graph),
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
	fn error_when_no_cell_ref() {
		let (mut app, _) = setup(Some(vec![]));

		app.world_mut().spawn(Grid::from(GridGraph::default()));

		assert_eq!(
			Some(&_Result(Err(GridError::NoRefToCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_valid_cell_ref() {
		let (mut app, _) = setup(None);

		app.world_mut().spawn((
			Grid::from(GridGraph::default()),
			CellsRef::<_Cell>::from_grid_definition(Entity::from_raw(123)),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::NoCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn grid_error_no_valid_map() {
		let (mut app, cell_definition) = setup(None);

		app.world_mut().spawn((
			Grid::from(GridGraph::default()),
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
		let (mut app, cell_definition) = setup(Some(cells));

		app.world_mut().spawn((
			Grid::from(graph),
			CellsRef::<_Cell>::from_grid_definition(cell_definition),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::GridIndexHasNoCell { x: 1, z: 0 }))),
			app.world().get_resource::<_Result>()
		);
	}
}
