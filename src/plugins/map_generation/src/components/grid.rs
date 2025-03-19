mod error_type_marker;

use crate::{
	grid_graph::GridGraph,
	traits::{
		GridCellDistanceDefinition,
		insert_cell_components::InsertCellComponents,
		to_subdivided::ToSubdivided,
	},
};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	errors::{Error, Level as ErrorLevel},
	traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn},
};
use error_type_marker::TypeMarker;
use std::any::type_name;

#[derive(Component, Debug, PartialEq)]
#[require(Name(Self::name), Transform, Visibility)]
pub struct Grid<const SUBDIVISIONS: u8 = 0, TGraph = GridGraph>
where
	TGraph: ToSubdivided,
{
	graph: TGraph,
}

impl Grid {
	pub(crate) fn spawn_cells<TCell, TError>(
		In(cells): In<Result<Vec<(Vec3, TCell)>, TError>>,
		mut commands: Commands,
		grids: Query<Entity, With<Self>>,
	) -> Result<(), SpawnCellError<TError, Self>>
	where
		TCell: InsertCellComponents + GridCellDistanceDefinition,
	{
		let cells = cells.map_err(|error| SpawnCellError::Error(error))?;
		let level = grids.get_single().map_err(|error| match error {
			QuerySingleError::NoEntities(_) => SpawnCellError::NoGrid,
			QuerySingleError::MultipleEntities(_) => SpawnCellError::MultipleGrids,
		})?;

		let Some(mut grid) = commands.get_entity(level) else {
			return Err(SpawnCellError::EntityCommandsError); // untested, don't know how to simulate
		};

		let scale = Vec3::splat(TCell::CELL_DISTANCE);
		grid.with_children(spawn_children(cells, scale));

		Ok(())
	}
}

impl<const SUBDIVISIONS: u8, TGraph> Grid<SUBDIVISIONS, TGraph>
where
	TGraph: ToSubdivided,
{
	fn name() -> String {
		format!("Grid (subdivisions: {SUBDIVISIONS})")
	}

	pub(crate) fn insert(
		mut commands: Commands,
		levels: Query<(Entity, &Grid<0, TGraph>), Changed<Grid<0, TGraph>>>,
	) where
		TGraph: ThreadSafe,
	{
		for (entity, level) in &levels {
			let graph = level.graph.to_subdivided(SUBDIVISIONS);
			commands.try_insert_on(entity, Self { graph });
		}
	}
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl From<GridGraph> for Grid {
	fn from(graph: GridGraph) -> Self {
		Grid { graph }
	}
}

impl<const SUBDIVISIONS: u8> From<&Grid<SUBDIVISIONS>> for GridGraph {
	fn from(value: &Grid<SUBDIVISIONS>) -> Self {
		value.graph.clone()
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum SpawnCellError<TError, TComponent> {
	Error(TError),
	NoGrid,
	MultipleGrids,
	EntityCommandsError,
	#[allow(private_interfaces)]
	_P(TypeMarker<TComponent>),
}

impl<TError, TComponent> From<SpawnCellError<TError, TComponent>> for Error
where
	Error: From<TError>,
{
	fn from(value: SpawnCellError<TError, TComponent>) -> Self {
		match value {
			SpawnCellError::Error(error) => Error::from(error),
			SpawnCellError::NoGrid => Error {
				msg: format!("No `{}` component exists", type_name::<TComponent>()),
				lvl: ErrorLevel::Error,
			},
			SpawnCellError::MultipleGrids => Error {
				msg: format!("Multiple `{}` components exist", type_name::<TComponent>()),
				lvl: ErrorLevel::Error,
			},
			SpawnCellError::EntityCommandsError => Error {
				msg: format!(
					"Failed to retrieve `{}` entity commands",
					type_name::<TComponent>()
				),
				lvl: ErrorLevel::Error,
			},
			SpawnCellError::_P(_) => unreachable!("Should not be possible to build"),
		}
	}
}

fn spawn_children<TCell>(cells: Vec<(Vec3, TCell)>, scale: Vec3) -> impl FnOnce(&mut ChildBuilder)
where
	TCell: InsertCellComponents + GridCellDistanceDefinition,
{
	move |parent| {
		for (translation, cell) in cells {
			let mut transform = Transform::from_translation(translation).with_scale(scale);
			if cell.offset_height() {
				transform.translation.y += TCell::CELL_DISTANCE / 2.;
			}
			let mut child = parent.spawn(transform);
			cell.insert_cell_components(&mut child);
		}
	}
}

#[cfg(test)]
mod test_insert_subdivided {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Graph {
		subdivisions: u8,
	}

	impl ToSubdivided for _Graph {
		fn to_subdivided(&self, subdivisions: u8) -> Self {
			_Graph { subdivisions }
		}
	}

	fn setup<const SUBDIVISIONS: u8>() -> App {
		let mut app = App::new();
		app.add_systems(Update, Grid::<SUBDIVISIONS, _Graph>::insert);

		app
	}

	#[test]
	fn spawn_subdivided() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Grid::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Grid {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Grid::<5, _Graph>>()
		);
	}

	#[test]
	fn do_not_insert_when_level_not_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Grid::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Grid<5, _Graph>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Grid::<5, _Graph>>());
	}

	#[test]
	fn insert_again_when_level_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Grid::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Grid<5, _Graph>>()
			.get_mut::<Grid<0, _Graph>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&Grid {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Grid::<5, _Graph>>()
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::insert_cell_components::InsertCellComponents;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{assert_count, get_children, test_tools::utils::SingleThreadedApp};

	#[derive(Default)]
	struct _Cell {
		offset_height: bool,
	}

	impl InsertCellComponents for _Cell {
		fn offset_height(&self) -> bool {
			self.offset_height
		}

		fn insert_cell_components(&self, entity: &mut EntityCommands) {
			entity.insert(_CellComponent);
		}
	}

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: f32 = 0.5;
	}

	#[derive(Component, Debug, PartialEq)]
	struct _CellComponent;

	#[derive(Debug, PartialEq)]
	struct _Error;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_transform_as_children_of_level() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(Grid::default()).id();
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![
			(
				Vec3::new(1., 2., 3.),
				_Cell {
					offset_height: true,
				},
			),
			(
				Vec3::new(3., 4., 5.),
				_Cell {
					offset_height: false,
				},
			),
			(
				Vec3::new(10., 21., 2.),
				_Cell {
					offset_height: true,
				},
			),
		]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Grid::spawn_cells)?;

		let entities = assert_count!(3, get_children!(app, grid));
		assert_eq!(
			[
				Some(&Transform::from_xyz(1., 2.25, 3.).with_scale(Vec3::splat(0.5))),
				Some(&Transform::from_xyz(3., 4., 5.).with_scale(Vec3::splat(0.5))),
				Some(&Transform::from_xyz(10., 21.25, 2.).with_scale(Vec3::splat(0.5))),
			],
			entities.map(|e| e.get::<Transform>())
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn spawn_cell_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(Grid::default()).id();
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![
			(Vec3::default(), _Cell::default()),
			(Vec3::default(), _Cell::default()),
			(Vec3::default(), _Cell::default()),
		]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Grid::spawn_cells)?;

		let entities = assert_count!(3, get_children!(app, grid));
		assert_eq!(
			[
				Some(&_CellComponent),
				Some(&_CellComponent),
				Some(&_CellComponent),
			],
			entities.map(|e| e.get::<_CellComponent>())
		);
		assert!(result.is_ok());
		Ok(())
	}

	#[test]
	fn return_cells_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Grid::default());
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Err(_Error);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Grid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::Error(_Error)), result);
		Ok(())
	}

	#[test]
	fn return_no_level_error() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _NotALevel;

		let mut app = setup();
		app.world_mut().spawn(_NotALevel);
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Grid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::NoGrid), result);
		Ok(())
	}

	#[test]
	fn return_multiple_levels_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Grid::default());
		app.world_mut().spawn(Grid::default());
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Grid::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::MultipleGrids), result);
		Ok(())
	}
}
