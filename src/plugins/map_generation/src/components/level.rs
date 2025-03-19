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

#[derive(Component, Debug, PartialEq)]
#[require(Name(Self::name), Transform, Visibility)]
pub struct Level<const SUBDIVISIONS: u8 = 0, TGraph = GridGraph>
where
	TGraph: ToSubdivided,
{
	graph: TGraph,
}

impl Level {
	pub(crate) fn spawn_cells<TCell, TError>(
		In(cells): In<Result<Vec<(Vec3, TCell)>, TError>>,
		mut commands: Commands,
		levels: Query<Entity, With<Self>>,
	) -> Result<(), SpawnCellError<TError>>
	where
		TCell: InsertCellComponents + GridCellDistanceDefinition,
	{
		let cells = cells.map_err(|error| SpawnCellError::Error(error))?;
		let level = levels.get_single().map_err(|error| match error {
			QuerySingleError::NoEntities(_) => SpawnCellError::NoLevel,
			QuerySingleError::MultipleEntities(_) => SpawnCellError::MultipleLevels,
		})?;

		let Some(mut level) = commands.get_entity(level) else {
			return Err(SpawnCellError::EntityCommandsError); // untested, don't know how to simulate
		};

		let scale = Vec3::splat(TCell::CELL_DISTANCE);
		level.with_children(spawn_children(cells, scale));

		Ok(())
	}
}

impl<const SUBDIVISIONS: u8, TGraph> Level<SUBDIVISIONS, TGraph>
where
	TGraph: ToSubdivided,
{
	fn name() -> String {
		format!("Level (subdivisions: {SUBDIVISIONS})")
	}

	pub(crate) fn insert(
		mut commands: Commands,
		levels: Query<(Entity, &Level<0, TGraph>), Changed<Level<0, TGraph>>>,
	) where
		TGraph: ThreadSafe,
	{
		for (entity, level) in &levels {
			let graph = level.graph.to_subdivided(SUBDIVISIONS);
			commands.try_insert_on(entity, Self { graph });
		}
	}
}

impl Default for Level {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl From<GridGraph> for Level {
	fn from(graph: GridGraph) -> Self {
		Level { graph }
	}
}

impl<const SUBDIVISIONS: u8> From<&Level<SUBDIVISIONS>> for GridGraph {
	fn from(value: &Level<SUBDIVISIONS>) -> Self {
		value.graph.clone()
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum SpawnCellError<TError> {
	Error(TError),
	NoLevel,
	MultipleLevels,
	EntityCommandsError,
}

impl<TError> From<SpawnCellError<TError>> for Error
where
	Error: From<TError>,
{
	fn from(value: SpawnCellError<TError>) -> Self {
		match value {
			SpawnCellError::Error(error) => Error::from(error),
			SpawnCellError::NoLevel => Error {
				msg: "No `Level` component exists".to_owned(),
				lvl: ErrorLevel::Error,
			},
			SpawnCellError::MultipleLevels => Error {
				msg: "Multiple `Level` components exist".to_owned(),
				lvl: ErrorLevel::Error,
			},
			SpawnCellError::EntityCommandsError => Error {
				msg: "Failed to retrieve `Level` entity commands".to_owned(),
				lvl: ErrorLevel::Error,
			},
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
		app.add_systems(Update, Level::<SUBDIVISIONS, _Graph>::insert);

		app
	}

	#[test]
	fn spawn_subdivided() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Level {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Level::<5, _Graph>>()
		);
	}

	#[test]
	fn do_not_insert_when_level_not_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Level<5, _Graph>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Level::<5, _Graph>>());
	}

	#[test]
	fn insert_again_when_level_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Level<5, _Graph>>()
			.get_mut::<Level<0, _Graph>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&Level {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Level::<5, _Graph>>()
		);
	}
}

#[cfg(test)]
mod tests {
	use crate::traits::insert_cell_components::InsertCellComponents;

	use super::*;
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
		let level = app.world_mut().spawn(Level::default()).id();
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
			.run_system_once_with(cells, Level::spawn_cells)?;

		let entities = assert_count!(3, get_children!(app, level));
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
		let level = app.world_mut().spawn(Level::default()).id();
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![
			(Vec3::default(), _Cell::default()),
			(Vec3::default(), _Cell::default()),
			(Vec3::default(), _Cell::default()),
		]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Level::spawn_cells)?;

		let entities = assert_count!(3, get_children!(app, level));
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
		app.world_mut().spawn(Level::default());
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Err(_Error);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Level::spawn_cells)?;

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
			.run_system_once_with(cells, Level::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::NoLevel), result);
		Ok(())
	}

	#[test]
	fn return_multiple_levels_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Level::default());
		app.world_mut().spawn(Level::default());
		let cells: Result<Vec<(Vec3, _Cell)>, _Error> = Ok(vec![]);

		let result = app
			.world_mut()
			.run_system_once_with(cells, Level::spawn_cells)?;

		assert_eq!(Err(SpawnCellError::MultipleLevels), result);
		Ok(())
	}
}
