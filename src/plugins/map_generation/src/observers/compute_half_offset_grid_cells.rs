use crate::{
	components::{
		cells_ref::CellsRef,
		half_offset_grid::HalfOffsetGrid,
		map::{
			cells::{CellGrid, MapCells, half_offset_cell::HalfOffsetCell},
			grid_graph::MapGridGraph,
		},
	},
	errors::GridError,
	traits::GridCellDistanceDefinition,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

pub(crate) type Cells<TCell> = (Entity, Vec<(Vec3, HalfOffsetCell<TCell>)>);

impl HalfOffsetGrid {
	pub(crate) fn compute_cells<TCell>(
		trigger: Trigger<OnAdd, Self>,
		grids: Query<&CellsRef<TCell>, With<Self>>,
		maps: Query<(&MapGridGraph<TCell>, &MapCells<TCell>)>,
	) -> Result<Cells<TCell>, GridError>
	where
		TCell: TypePath + ThreadSafe + GridCellDistanceDefinition + Clone,
	{
		let entity = trigger.target();
		let Ok(cells_ref) = grids.get(entity) else {
			return Err(GridError::NoRefToCellDefinition);
		};
		let Ok((graph, map)) = maps.get(cells_ref.cell_definition) else {
			return Err(GridError::NoCellDefinition);
		};
		let graph = graph.graph();
		let offset = *TCell::CELL_DISTANCE / 2.;
		let mut index_mismatch = vec![];
		let mut cells = vec![];
		let CellGrid(half_offset_cells) = &map.half_offset_cells;

		for x in 0..(*map.size.x - 1) {
			for z in 0..(*map.size.z - 1) {
				let Some(translation) = graph.nodes.get(&(x, z)) else {
					continue;
				};
				let Some(cell) = half_offset_cells.get(&(x, z)) else {
					index_mismatch.push((x, z));
					continue;
				};
				cells.push((translation + Vec3::new(offset, 0., offset), cell.clone()));
			}
		}

		match index_mismatch.as_slice() {
			[] => Ok((entity, cells)),
			_ => Err(GridError::GridIndicesWithNoCell(index_mismatch)),
		}
	}
}

#[cfg(test)]
mod test_get_half_offset_grid {
	use super::*;
	use crate::{
		cell_grid_size::CellGridSize,
		components::{cells_ref::CellsRef, map::cells::Direction},
		grid_graph::{
			GridGraph,
			grid_context::{CellCount, CellDistance},
		},
		traits::GridCellDistanceDefinition,
	};
	use macros::new_valid;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_eq_unordered};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(&'static str);

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 2.);
	}

	#[derive(Resource, Debug, PartialEq, Clone)]
	struct _Result(Result<Cells<_Cell>, GridError>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(HalfOffsetGrid::compute_cells::<_Cell>.pipe(
			|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			},
		));

		app
	}

	#[test]
	fn return_grid_cells_with_translation() {
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn((
				MapGridGraph::<_Cell>::from(GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(-1., -0., -1.)),
						((0, 1), Vec3::new(-1., -0., 1.)),
						((1, 0), Vec3::new(1., -0., -1.)),
						((1, 1), Vec3::new(1., -0., 1.)),
					]),
					..default()
				}),
				MapCells {
					size: CellGridSize {
						x: new_valid!(CellCount, 2),
						z: new_valid!(CellCount, 2),
					},
					half_offset_cells: CellGrid::from([(
						(0, 0),
						HalfOffsetCell::from([
							(Direction::Z, _Cell("00")),
							(Direction::NegX, _Cell("01")),
							(Direction::X, _Cell("10")),
							(Direction::NegZ, _Cell("11")),
						]),
					)]),
					..default()
				},
			))
			.id();

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cells),
		));

		assert_eq_unordered!(
			Ok(vec![(
				Vec3::new(0., 0., 0.),
				HalfOffsetCell::from([
					(Direction::Z, _Cell("00")),
					(Direction::NegX, _Cell("01")),
					(Direction::X, _Cell("10")),
					(Direction::NegZ, _Cell("11")),
				])
			)]),
			app.world()
				.resource::<_Result>()
				.0
				.clone()
				.map(|(_, cells)| cells)
		);
	}

	#[test]
	fn return_grid_cells_with_non_zero_translation() {
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn((
				MapGridGraph::<_Cell>::from(GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(-2., -0., -1.)),
						((0, 1), Vec3::new(-2., -0., 1.)),
						((1, 0), Vec3::new(0., -0., -1.)),
						((1, 1), Vec3::new(0., -0., 1.)),
						((2, 0), Vec3::new(2., -0., -1.)),
						((2, 1), Vec3::new(2., -0., 1.)),
					]),
					..default()
				}),
				MapCells {
					size: CellGridSize {
						x: new_valid!(CellCount, 3),
						z: new_valid!(CellCount, 2),
					},
					half_offset_cells: CellGrid::from([
						(
							(0, 0),
							HalfOffsetCell::from([
								(Direction::Z, _Cell("a 00")),
								(Direction::NegX, _Cell("a 01")),
								(Direction::X, _Cell("a 10")),
								(Direction::NegZ, _Cell("a 11")),
							]),
						),
						(
							(1, 0),
							HalfOffsetCell::from([
								(Direction::Z, _Cell("b 00")),
								(Direction::NegX, _Cell("b 01")),
								(Direction::X, _Cell("b 10")),
								(Direction::NegZ, _Cell("b 11")),
							]),
						),
					]),
					..default()
				},
			))
			.id();

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cells),
		));

		assert_eq_unordered!(
			Ok(vec![
				(
					Vec3::new(-1., 0., 0.),
					HalfOffsetCell::from([
						(Direction::Z, _Cell("a 00")),
						(Direction::NegX, _Cell("a 01")),
						(Direction::X, _Cell("a 10")),
						(Direction::NegZ, _Cell("a 11")),
					]),
				),
				(
					Vec3::new(1., 0., 0.),
					HalfOffsetCell::from([
						(Direction::Z, _Cell("b 00")),
						(Direction::NegX, _Cell("b 01")),
						(Direction::X, _Cell("b 10")),
						(Direction::NegZ, _Cell("b 11")),
					]),
				),
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
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn((
				MapGridGraph::<_Cell>::from(GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(-1., -0., -1.)),
						((0, 1), Vec3::new(-1., -0., 1.)),
						((1, 0), Vec3::new(1., -0., -1.)),
						((1, 1), Vec3::new(1., -0., 1.)),
					]),
					..default()
				}),
				MapCells {
					size: CellGridSize {
						x: new_valid!(CellCount, 2),
						z: new_valid!(CellCount, 2),
					},
					half_offset_cells: CellGrid::from([(
						(0, 0),
						HalfOffsetCell::from([
							(Direction::Z, _Cell("00")),
							(Direction::NegX, _Cell("01")),
							(Direction::X, _Cell("10")),
							(Direction::NegZ, _Cell("11")),
						]),
					)]),
					..default()
				},
			))
			.id();

		let grid = app
			.world_mut()
			.spawn((
				HalfOffsetGrid,
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
	fn grid_error_graph_index_has_no_cell() {
		let mut app = setup();
		let cells = app
			.world_mut()
			.spawn((
				MapGridGraph::<_Cell>::from(GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(-2., -0., -1.)),
						((0, 1), Vec3::new(-2., -0., 1.)),
						((1, 0), Vec3::new(0., -0., -1.)),
						((1, 1), Vec3::new(0., -0., 1.)),
						((2, 0), Vec3::new(2., -0., -1.)),
						((2, 1), Vec3::new(2., -0., 1.)),
					]),
					..default()
				}),
				MapCells {
					size: CellGridSize {
						x: new_valid!(CellCount, 3),
						z: new_valid!(CellCount, 2),
					},
					half_offset_cells: CellGrid::from([(
						(0, 0),
						HalfOffsetCell::from([
							(Direction::Z, _Cell("00")),
							(Direction::NegX, _Cell("01")),
							(Direction::X, _Cell("10")),
							(Direction::NegZ, _Cell("11")),
						]),
					)]),
					..default()
				},
			))
			.id();

		app.world_mut().spawn((
			HalfOffsetGrid,
			CellsRef::<_Cell>::from_grid_definition(cells),
		));

		assert_eq!(
			Some(&_Result(Err(GridError::GridIndicesWithNoCell(vec![(
				1, 0
			)])))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_cell_ref() {
		let mut app = setup();

		app.world_mut().spawn(HalfOffsetGrid);

		assert_eq!(
			Some(&_Result(Err(GridError::NoRefToCellDefinition))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn error_when_no_valid_cell_ref() {
		let mut app = setup();

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
