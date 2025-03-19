use super::grid::SpawnCellError;
use crate::{
	grid_graph::GridGraph,
	map::{Direction, half_offset_cell::HalfOffsetCell},
	traits::{
		insert_cell_quadrant_components::{InsertCellQuadrantComponents, Quadrant},
		is_walkable::IsWalkable,
	},
};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
#[require(Name(Self::name))]
pub(crate) struct HalfOffsetGrid;

impl HalfOffsetGrid {
	fn name() -> String {
		"HalfOffsetGrid".to_owned()
	}

	#[allow(clippy::type_complexity)]
	pub(crate) fn spawn_cells<TCell, TError>(
		In(cells): In<Result<Vec<(Vec3, HalfOffsetCell<TCell>)>, TError>>,
		mut commands: Commands,
		grids: Query<Entity, With<Self>>,
	) -> Result<(), SpawnCellError<TError, Self>>
	where
		TCell: InsertCellQuadrantComponents + IsWalkable,
	{
		let cells = cells.map_err(SpawnCellError::Error)?;
		let level = grids.get_single().map_err(|error| match error {
			QuerySingleError::NoEntities(_) => SpawnCellError::NoGrid,
			QuerySingleError::MultipleEntities(_) => SpawnCellError::MultipleGrids,
		})?;

		let Some(mut grid) = commands.get_entity(level) else {
			return Err(SpawnCellError::EntityCommandsError); // untested, don't know how to simulate
		};

		grid.with_children(spawn_children(cells));

		Ok(())
	}
}

impl From<GridGraph> for HalfOffsetGrid {
	fn from(_: GridGraph) -> Self {
		Self
	}
}

fn spawn_children<TCell>(
	cells: Vec<(Vec3, HalfOffsetCell<TCell>)>,
) -> impl FnOnce(&mut ChildBuilder)
where
	TCell: InsertCellQuadrantComponents + IsWalkable,
{
	|parent| {
		for (translation, cell) in cells {
			spawn_quadrant(parent, translation, cell);
		}
	}
}

fn spawn_quadrant<TCell>(parent: &mut ChildBuilder, pos: Vec3, cell: HalfOffsetCell<TCell>)
where
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
		let mut child = parent.spawn(transform);
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
	use crate::{map::Direction, traits::insert_cell_quadrant_components::Quadrant};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		assert_count,
		assert_eq_unordered,
		get_children,
		test_tools::utils::SingleThreadedApp,
	};
	use std::collections::HashSet;

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
			entity: &mut EntityCommands,
			pattern: HashSet<Quadrant>,
		) {
			entity.insert(_Quadrant {
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
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Ok(vec![(
			Vec3::new(1., 2., 3.),
			HalfOffsetCell::from([
				(Direction::NegZ, _Cell::walkable("")),
				(Direction::X, _Cell::walkable("")),
				(Direction::Z, _Cell::walkable("")),
				(Direction::NegX, _Cell::walkable("")),
			]),
		)]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;
		let children = assert_count!(4, get_children!(app, grid));
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
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Ok(vec![(
			Vec3::new(1., 2., 3.),
			HalfOffsetCell::from([
				(Direction::NegZ, _Cell::walkable("neg z")),
				(Direction::X, _Cell::walkable("x")),
				(Direction::Z, _Cell::walkable("z")),
				(Direction::NegX, _Cell::walkable("neg x")),
			]),
		)]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;
		let children = assert_count!(4, get_children!(app, grid));
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
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Ok(vec![(
			Vec3::new(1., 2., 3.),
			HalfOffsetCell::from([
				(Direction::X, _Cell::walkable("x")),
				(Direction::Z, _Cell::walkable("z")),
				(Direction::NegX, _Cell::walkable("neg x")),
				(Direction::NegZ, _Cell::not_walkable("neg z")),
			]),
		)]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;
		let children = assert_count!(4, get_children!(app, grid));
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
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Err(_Error);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::Error(_Error)), result);
		Ok(())
	}

	#[test]
	fn return_no_level_error() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _NotALevel;

		let mut app = setup();
		app.world_mut().spawn(_NotALevel);
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Ok(vec![]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::NoGrid), result);
		Ok(())
	}

	#[test]
	fn return_multiple_levels_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(HalfOffsetGrid);
		app.world_mut().spawn(HalfOffsetGrid);
		let cells: Result<Vec<(Vec3, HalfOffsetCell<_Cell>)>, _Error> = Ok(vec![]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, HalfOffsetGrid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::MultipleGrids), result);
		Ok(())
	}
}
