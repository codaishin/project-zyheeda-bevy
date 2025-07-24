use crate::{
	components::{
		cells_ref::CellsRef,
		grid::Grid,
		map::cells::{CellGrid, MapCells},
	},
	errors::GridError,
	traits::map_cells_extra::MapCellsExtra,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

pub(crate) type Cells<TCell> = (Entity, Vec<(Vec3, TCell)>);

impl Grid {
	pub(crate) fn compute_cells<TCell>(
		trigger: Trigger<OnAdd, Self>,
		grids: Query<(&Self, &CellsRef<TCell>)>,
		cells: Query<&MapCells<TCell>>,
	) -> Result<Cells<TCell>, GridError>
	where
		TCell: TypePath + ThreadSafe + Clone + MapCellsExtra,
	{
		let entity = trigger.target();
		let Ok((grid, cells_ref)) = grids.get(entity) else {
			return Err(GridError::NoRefToCellDefinition);
		};
		let Ok(MapCells { definition, .. }) = cells.get(cells_ref.cell_definition) else {
			return Err(GridError::NoCellDefinition);
		};
		let CellGrid(cells) = &definition.cells;

		let mut index_mismatch = None;
		let cell_translation = |((x, z), translation): (&(u32, u32), &Vec3)| {
			let x = *x;
			let z = *z;
			let Some(cell) = cells.get(&(x, z)) else {
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
			return Err(GridError::GridIndicesWithNoCell(vec![(x, z)]));
		}

		Ok((entity, cells))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::map::cells::CellGrid,
		grid_graph::GridGraph,
		traits::map_cells_extra::{CellGridDefinition, MapCellsExtra},
	};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_eq_unordered};

	#[derive(Resource, Debug, PartialEq, Clone)]
	struct _Result(Result<(Entity, Vec<(Vec3, _Cell)>), GridError>);

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	impl MapCellsExtra for _Cell {
		type TExtra = ();
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(
			Grid::compute_cells.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			}),
		);

		app
	}

	#[test]
	fn return_grid_cells_with_translation() {
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn(MapCells {
				definition: CellGridDefinition {
					cells: CellGrid::from([
						((0, 0), _Cell("00")),
						((0, 1), _Cell("01")),
						((1, 0), _Cell("10")),
						((1, 1), _Cell("11")),
					]),
					..default()
				},
				..default()
			})
			.id();

		app.world_mut().spawn((
			Grid::from(&GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-1., -0., -1.)),
					((0, 1), Vec3::new(-1., -0., 1.)),
					((1, 0), Vec3::new(1., -0., -1.)),
					((1, 1), Vec3::new(1., -0., 1.)),
				]),
				..default()
			}),
			CellsRef::<_Cell>::from_grid_definition(cells),
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
			..default()
		};
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn(MapCells {
				definition: CellGridDefinition {
					cells: CellGrid::from([
						((0, 0), _Cell("00")),
						((0, 1), _Cell("01")),
						((1, 0), _Cell("10")),
						((1, 1), _Cell("11")),
					]),
					..default()
				},
				..default()
			})
			.id();

		let grid = app
			.world_mut()
			.spawn((
				Grid::from(&graph),
				CellsRef::<_Cell>::from_grid_definition(cells),
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
		let mut app = setup();

		app.world_mut().spawn(Grid::from(&GridGraph::default()));

		assert_eq!(
			Some(&_Result(Err(GridError::NoRefToCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_valid_cell_ref() {
		let mut app = setup();

		app.world_mut().spawn((
			Grid::from(&GridGraph::default()),
			CellsRef::<_Cell>::from_grid_definition(Entity::from_raw(123)),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::NoCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn grid_error_graph_index_has_no_cell() {
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn(MapCells {
				definition: CellGridDefinition {
					cells: CellGrid::from([
						((0, 0), _Cell("00")),
						((0, 1), _Cell("01")),
						((1, 1), _Cell("11")),
					]),
					..default()
				},
				..default()
			})
			.id();

		app.world_mut().spawn((
			Grid::from(&GridGraph {
				nodes: HashMap::from([
					((0, 0), Vec3::new(-1., -0., -1.)),
					((0, 1), Vec3::new(-1., -0., 1.)),
					((1, 0), Vec3::new(1., -0., -1.)),
					((1, 1), Vec3::new(1., -0., 1.)),
				]),
				..default()
			}),
			CellsRef::<_Cell>::from_grid_definition(cells),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::GridIndicesWithNoCell(vec![(
				1, 0
			)])))),
			app.world().get_resource::<_Result>()
		);
	}
}
