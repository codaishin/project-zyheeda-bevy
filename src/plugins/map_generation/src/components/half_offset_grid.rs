use super::grid::SpawnCellError;
use crate::{
	components::map::cells::{Direction, half_offset_cell::HalfOffsetCell},
	grid_graph::GridGraph,
	observers::compute_half_offset_grid_cells::Cells,
	traits::{
		insert_cell_quadrant_components::{InsertCellQuadrantComponents, Quadrant},
		is_walkable::IsWalkable,
	},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::zyheeda_commands::ZyheedaEntityCommands;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
#[require(Name = "HalfOffsetGrid", Transform, Visibility)]
pub(crate) struct HalfOffsetGrid;

impl HalfOffsetGrid {
	pub(crate) fn spawn_cells<TCell, TError>(
		In(cells): In<Result<Cells<TCell>, TError>>,
		mut commands: Commands,
	) -> Result<(), SpawnCellError<TError, Self>>
	where
		TCell: InsertCellQuadrantComponents + IsWalkable,
	{
		let (entity, cells) = cells.map_err(SpawnCellError::Error)?;
		let Ok(mut grid) = commands.get_entity(entity) else {
			return Err(SpawnCellError::NoGridEntity);
		};

		grid.with_children(spawn_children(cells));

		Ok(())
	}
}

impl From<&GridGraph> for HalfOffsetGrid {
	fn from(_: &GridGraph) -> Self {
		Self
	}
}

fn spawn_children<TCell>(
	cells: Vec<(Vec3, HalfOffsetCell<TCell>)>,
) -> impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>)
where
	TCell: InsertCellQuadrantComponents + IsWalkable,
{
	|parent| {
		for (translation, cell) in cells {
			spawn_quadrant(parent, translation, cell);
		}
	}
}

fn spawn_quadrant<TCell>(
	parent: &mut RelatedSpawnerCommands<ChildOf>,
	pos: Vec3,
	cell: HalfOffsetCell<TCell>,
) where
	TCell: InsertCellQuadrantComponents + IsWalkable,
{
	let quadrants = cell.quadrants();

	for (look_dir, cell) in quadrants {
		let walk_ability = cell.is_walkable();
		let neighbor_quadrants = [
			(look_dir.rotated_right(1), Quadrant::Forward),
			(look_dir.rotated_right(2), Quadrant::Diagonal),
			(look_dir.rotated_right(3), Quadrant::Left),
		];

		let different_quadrants = neighbor_quadrants
			.into_iter()
			.filter_map(different_quadrant(quadrants, walk_ability))
			.collect();

		let transform = Transform::from_translation(pos).looking_to(Dir3::from(*look_dir), Vec3::Y);
		let mut child = ZyheedaEntityCommands::from(parent.spawn(transform));
		cell.insert_cell_quadrant_components(&mut child, different_quadrants);
	}
}

fn different_quadrant<TCell>(
	quadrants: &HashMap<Direction, TCell>,
	walk_ability: bool,
) -> impl Fn((Direction, Quadrant)) -> Option<Quadrant>
where
	TCell: IsWalkable,
{
	move |(dir, quadrant)| {
		let quadrant_cell = quadrants.get(&dir)?;
		if quadrant_cell.is_walkable() == walk_ability {
			return None;
		}
		Some(quadrant)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::insert_cell_quadrant_components::Quadrant;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, assert_children_count, assert_eq_unordered, fake_entity};

	struct _Cell {
		name: &'static str,
		is_walkable: bool,
	}

	impl _Cell {
		fn walkable(name: &'static str) -> Self {
			Self {
				is_walkable: true,
				name,
			}
		}

		fn not_walkable(name: &'static str) -> Self {
			Self {
				is_walkable: false,
				name,
			}
		}
	}

	impl InsertCellQuadrantComponents for _Cell {
		fn insert_cell_quadrant_components(
			&self,
			entity: &mut ZyheedaEntityCommands,
			pattern: HashSet<Quadrant>,
		) {
			entity.try_insert(_Quadrant {
				cell: self.name,
				differences: pattern,
			});
		}
	}

	impl IsWalkable for _Cell {
		fn is_walkable(&self) -> bool {
			self.is_walkable
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Quadrant {
		cell: &'static str,
		differences: HashSet<Quadrant>,
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_cell_quadrant_transforms() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(HalfOffsetGrid).id();
		let cells: Result<Cells<_Cell>, _Error> = Ok((
			grid,
			vec![(
				Vec3::new(1., 2., 3.),
				HalfOffsetCell::from([
					(Direction::NegZ, _Cell::walkable("")),
					(Direction::X, _Cell::walkable("")),
					(Direction::Z, _Cell::walkable("")),
					(Direction::NegX, _Cell::walkable("")),
				]),
			)],
		));

		let result = app
			.world_mut()
			.run_system_once_with(HalfOffsetGrid::spawn_cells, cells)?;
		let children = assert_children_count!(4, app, grid);
		assert_eq_unordered!(
			[
				Some(&Transform::from_xyz(1., 2., 3.).looking_to(Dir3::NEG_Z, Vec3::Y)),
				Some(&Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Vec3::Y)),
				Some(&Transform::from_xyz(1., 2., 3.).looking_to(Dir3::Z, Vec3::Y)),
				Some(&Transform::from_xyz(1., 2., 3.).looking_to(Dir3::NEG_X, Vec3::Y)),
			],
			children.map(|e| e.get::<Transform>())
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn spawn_cell_quadrants_all_walkable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(HalfOffsetGrid).id();
		let cells: Result<Cells<_Cell>, _Error> = Ok((
			grid,
			vec![(
				Vec3::new(1., 2., 3.),
				HalfOffsetCell::from([
					(Direction::NegZ, _Cell::walkable("neg z")),
					(Direction::X, _Cell::walkable("x")),
					(Direction::Z, _Cell::walkable("z")),
					(Direction::NegX, _Cell::walkable("neg x")),
				]),
			)],
		));

		let result = app
			.world_mut()
			.run_system_once_with(HalfOffsetGrid::spawn_cells, cells)?;
		let children = assert_children_count!(4, app, grid);
		assert_eq_unordered!(
			[
				Some(&_Quadrant {
					cell: "neg z",
					differences: HashSet::from([])
				}),
				Some(&_Quadrant {
					cell: "z",
					differences: HashSet::from([])
				}),
				Some(&_Quadrant {
					cell: "neg x",
					differences: HashSet::from([])
				}),
				Some(&_Quadrant {
					cell: "x",
					differences: HashSet::from([])
				})
			],
			children.map(|e| e.get::<_Quadrant>())
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn spawn_cell_corner() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(HalfOffsetGrid).id();
		let cells: Result<Cells<_Cell>, _Error> = Ok((
			grid,
			vec![(
				Vec3::new(1., 2., 3.),
				HalfOffsetCell::from([
					(Direction::X, _Cell::walkable("x")),
					(Direction::Z, _Cell::walkable("z")),
					(Direction::NegX, _Cell::walkable("neg x")),
					(Direction::NegZ, _Cell::not_walkable("neg z")),
				]),
			)],
		));

		let result = app
			.world_mut()
			.run_system_once_with(HalfOffsetGrid::spawn_cells, cells)?;
		let children = assert_children_count!(4, app, grid);
		assert_eq_unordered!(
			[
				Some(&_Quadrant {
					cell: "neg z",
					differences: HashSet::from([
						Quadrant::Diagonal,
						Quadrant::Forward,
						Quadrant::Left
					])
				}),
				Some(&_Quadrant {
					cell: "z",
					differences: HashSet::from([Quadrant::Diagonal])
				}),
				Some(&_Quadrant {
					cell: "neg x",
					differences: HashSet::from([Quadrant::Left])
				}),
				Some(&_Quadrant {
					cell: "x",
					differences: HashSet::from([Quadrant::Forward])
				})
			],
			children.map(|e| e.get::<_Quadrant>())
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn return_cells_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(HalfOffsetGrid);
		let mut app = setup();
		let cells: Result<Cells<_Cell>, _Error> = Ok((
			fake_entity!(123),
			vec![(
				Vec3::new(1., 2., 3.),
				HalfOffsetCell::from([
					(Direction::X, _Cell::walkable("x")),
					(Direction::Z, _Cell::walkable("z")),
					(Direction::NegX, _Cell::walkable("neg x")),
					(Direction::NegZ, _Cell::not_walkable("neg z")),
				]),
			)],
		));

		let result = app
			.world_mut()
			.run_system_once_with(HalfOffsetGrid::spawn_cells, cells)?;

		assert_eq!(Err(SpawnCellError::NoGridEntity), result);
		Ok(())
	}

	#[test]
	fn return_no_entity_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(HalfOffsetGrid);
		let cells: Result<Cells<_Cell>, _Error> = Err(_Error);

		let result = app
			.world_mut()
			.run_system_once_with(HalfOffsetGrid::spawn_cells, cells)?;

		assert_eq!(Err(SpawnCellError::Error(_Error)), result);
		Ok(())
	}
}
