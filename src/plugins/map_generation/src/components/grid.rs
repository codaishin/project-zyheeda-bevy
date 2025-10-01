use crate::{
	grid_graph::GridGraph,
	observers::compute_grid_cells::Cells,
	traits::{
		GridCellDistanceDefinition,
		insert_cell_components::InsertCellComponents,
		to_subdivided::{SubdivisionError, ToSubdivided},
	},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::{ErrorData, Level as ErrorLevel, Unreachable},
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use std::{any::type_name, fmt::Display, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
#[require(Name = Self::name(), Transform, Visibility, Sensor)]
pub struct Grid<const SUBDIVISIONS: u8 = 0, TGraph = GridGraph>
where
	TGraph: ToSubdivided,
{
	graph: TGraph,
}

impl Grid {
	pub(crate) fn graph(&self) -> &GridGraph {
		&self.graph
	}

	pub(crate) fn spawn_cells<TCell, TError>(
		In(cells): In<Result<Cells<TCell>, TError>>,
		mut commands: Commands,
	) -> Result<(), SpawnCellError<TError, Self>>
	where
		TCell: InsertCellComponents + GridCellDistanceDefinition,
	{
		let (entity, cells) = cells.map_err(|error| SpawnCellError::Error(error))?;
		let Ok(mut grid) = commands.get_entity(entity) else {
			return Err(SpawnCellError::NoGridEntity);
		};

		let scale = Vec3::splat(*TCell::CELL_DISTANCE);
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
		mut commands: ZyheedaCommands,
		levels: Query<(Entity, &Grid<0, TGraph>), Changed<Grid<0, TGraph>>>,
	) -> Result<(), Vec<SubdivisionError>>
	where
		TGraph: ThreadSafe,
	{
		let errors = levels
			.iter()
			.filter_map(
				|(entity, level)| match level.graph.to_subdivided(SUBDIVISIONS) {
					Ok(graph) => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(Self { graph });
						});
						None
					}
					Err(err) => Some(err),
				},
			)
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl From<&GridGraph> for Grid {
	fn from(graph: &GridGraph) -> Self {
		Grid {
			graph: graph.clone(),
		}
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
	NoGridEntity,
	_P((PhantomData<TComponent>, Unreachable)),
}

impl<TError, TComponent> Display for SpawnCellError<TError, TComponent>
where
	TError: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SpawnCellError::Error(error) => write!(f, "{error}"),
			SpawnCellError::NoGridEntity => write!(
				f,
				"Failed to retrieve `{}` entity commands",
				type_name::<TComponent>()
			),
			SpawnCellError::_P(_) => unreachable!(),
		}
	}
}

impl<TError, TComponent> ErrorData for SpawnCellError<TError, TComponent>
where
	TError: Display,
{
	fn level(&self) -> ErrorLevel {
		ErrorLevel::Error
	}

	fn label() -> impl Display {
		"Failed to spawn cell"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

fn spawn_children<TCell>(
	cells: Vec<(Vec3, TCell)>,
	scale: Vec3,
) -> impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>)
where
	TCell: InsertCellComponents + GridCellDistanceDefinition,
{
	move |parent| {
		for (translation, cell) in cells {
			let mut transform = Transform::from_translation(translation).with_scale(scale);
			if cell.offset_height() {
				transform.translation.y += *TCell::CELL_DISTANCE / 2.;
			}
			let mut child = ZyheedaEntityCommands::from(parent.spawn(transform));
			cell.insert_cell_components(&mut child);
		}
	}
}

#[cfg(test)]
mod test_insert_subdivided {
	use super::*;
	use crate::grid_graph::grid_context::DividedToZero;

	#[derive(Debug, PartialEq)]
	struct _Graph {
		subdivisions: u8,
	}

	const TO_ZERO_SUBDIVISION: u8 = 42;

	impl ToSubdivided for _Graph {
		fn to_subdivided(&self, subdivisions: u8) -> Result<Self, SubdivisionError> {
			if subdivisions == TO_ZERO_SUBDIVISION {
				return Err(SubdivisionError::CellDistanceZero(DividedToZero {
					from: 123.,
					divisor: TO_ZERO_SUBDIVISION,
				}));
			}

			Ok(_Graph { subdivisions })
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<SubdivisionError>>);

	fn setup<const SUBDIVISIONS: u8>() -> App {
		let mut app = App::new();
		app.add_systems(
			Update,
			Grid::<SUBDIVISIONS, _Graph>::insert.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result));
			}),
		);

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

	#[test]
	fn return_errors() {
		let mut app = setup::<TO_ZERO_SUBDIVISION>();
		app.world_mut().spawn(Grid::<0, _Graph> {
			graph: _Graph { subdivisions: 0 },
		});

		app.update();

		assert_eq!(
			Some(&_Result(Err(vec![SubdivisionError::CellDistanceZero(
				DividedToZero {
					from: 123.,
					divisor: TO_ZERO_SUBDIVISION,
				}
			)]))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_no_error_when_ok() {
		let mut app = setup::<5>();
		app.world_mut().spawn(Grid::<0, _Graph> {
			graph: _Graph { subdivisions: 0 },
		});

		app.update();

		assert_eq!(
			Some(&_Result(Ok(()))),
			app.world().get_resource::<_Result>()
		);
	}
}

#[cfg(test)]
mod test_spawn_cells {
	use super::*;
	use crate::{
		grid_graph::grid_context::CellDistance,
		traits::insert_cell_components::InsertCellComponents,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::new_valid;
	use testing::{SingleThreadedApp, assert_count, get_children};

	#[derive(Default)]
	struct _Cell {
		offset_height: bool,
	}

	impl InsertCellComponents for _Cell {
		fn offset_height(&self) -> bool {
			self.offset_height
		}

		fn insert_cell_components(&self, entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_CellComponent);
		}
	}

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 0.5);
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
		let cells: Result<Cells<_>, _Error> = Ok((
			grid,
			vec![
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
			],
		));

		let result = app
			.world_mut()
			.run_system_once_with(Grid::spawn_cells, cells)?;

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
		let cells: Result<Cells<_>, _Error> = Ok((
			grid,
			vec![
				(Vec3::default(), _Cell::default()),
				(Vec3::default(), _Cell::default()),
				(Vec3::default(), _Cell::default()),
			],
		));

		let result = app
			.world_mut()
			.run_system_once_with(Grid::spawn_cells, cells)?;

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
		let cells: Result<Cells<_Cell>, _Error> = Err(_Error);

		let result = app
			.world_mut()
			.run_system_once_with(Grid::spawn_cells, cells)?;

		assert_eq!(Err(SpawnCellError::Error(_Error)), result);
		Ok(())
	}

	#[test]
	fn return_commands_error_when_entity_not_valid() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Grid::default());
		let cells: Result<Cells<_Cell>, _Error> = Ok((Entity::from_raw(111), vec![]));

		let result = app
			.world_mut()
			.run_system_once_with(Grid::spawn_cells, cells)?;

		assert_eq!(Err(SpawnCellError::NoGridEntity), result);
		Ok(())
	}
}
