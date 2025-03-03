use crate::{
	grid_graph::{
		GridGraph,
		grid_context::{GridContext, GridDefinition},
	},
	map::Map,
	traits::{GridCellDistanceDefinition, SourcePath, grid_start::GridStart},
};
use bevy::prelude::*;
use common::traits::{load_asset::LoadAsset, thread_safe::ThreadSafe};
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoadLevel<TCell>(pub Handle<Map<TCell>>)
where
	TCell: TypePath + ThreadSafe;

impl<TCell> LoadLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn start(commands: Commands, map_loader: ResMut<AssetServer>)
	where
		TCell: SourcePath + TypePath + Sync + Send,
	{
		begin_level_load::<AssetServer, TCell>(commands, map_loader);
	}

	pub(crate) fn graph(
		mut commands: Commands,
		maps: Res<Assets<Map<TCell>>>,
		load_level_cmd: Option<Res<LoadLevel<TCell>>>,
	) -> Option<GridGraph<(Transform, TCell), ()>>
	where
		TCell: GridCellDistanceDefinition + Clone,
		for<'a> Dir3: From<&'a TCell>,
	{
		let cells = get_map_cells(load_level_cmd, maps)?;
		let (cell_count_x, cell_count_z) = get_cell_counts(&cells)?;
		let grid_definition = GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance: TCell::CELL_DISTANCE,
		};
		let Ok(context) = GridContext::try_from(grid_definition) else {
			return None;
		};
		let mut graph = GridGraph {
			nodes: HashMap::default(),
			extra: (),
			context,
		};

		commands.remove_resource::<LoadLevel<TCell>>();

		let min = context.grid_min();
		let mut position = min;
		let cell_distance = TCell::CELL_DISTANCE as f32;

		for (z, cell_line) in cells.into_iter().enumerate() {
			for (x, cell) in cell_line.into_iter().enumerate() {
				graph
					.nodes
					.insert((x as i32, z as i32), (transform(&cell, position), cell));
				position.x += cell_distance;
			}
			position.x = min.x;
			position.z += cell_distance;
		}

		Some(graph)
	}
}

pub(crate) fn begin_level_load<TLoadMap, TCell>(
	mut commands: Commands,
	mut map_loader: ResMut<TLoadMap>,
) where
	TLoadMap: LoadAsset + Resource,
	TCell: SourcePath + TypePath + Sync + Send,
{
	let map: Handle<Map<TCell>> = map_loader.load_asset(TCell::source_path());
	commands.insert_resource(LoadLevel(map));
}

fn get_map_cells<TCell>(
	load_level_cmd: Option<Res<LoadLevel<TCell>>>,
	maps: Res<Assets<Map<TCell>>>,
) -> Option<Vec<Vec<TCell>>>
where
	TCell: GridCellDistanceDefinition + TypePath + ThreadSafe + Clone,
{
	let map_handle = &load_level_cmd?.0;
	let map = maps.get(map_handle)?;

	Some(map.0.clone())
}

fn transform<TCell>(cell: &TCell, position: Vec3) -> Transform
where
	for<'a> Dir3: From<&'a TCell>,
{
	let direction = Dir3::from(cell);

	Transform::from_translation(position).looking_to(direction, Vec3::Y)
}

fn get_cell_counts<T>(cells: &[Vec<T>]) -> Option<(usize, usize)> {
	let count_x = cells.iter().map(|line| line.len()).max()?;
	let count_z = cells.len();

	Some((count_x, count_z))
}

#[cfg(test)]
mod test_begin_level_load {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Cell;

	impl SourcePath for _Cell {
		fn source_path() -> Path {
			Path::from("aaa/bbb/ccc.file_format")
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LoadMap {
		mock: Mock_LoadMap,
	}

	#[automock]
	impl LoadAsset for _LoadMap {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(load_map);
		app.add_systems(Update, begin_level_load::<_LoadMap, _Cell>);

		app
	}

	#[test]
	fn insert_level_command() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(handle.clone());
		}));

		app.update();

		let level_command = app.world().get_resource::<LoadLevel<_Cell>>();

		assert_eq!(Some(&LoadLevel(handle)), level_command);
	}
}

#[cfg(test)]
mod test_get_graph {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::Handle,
		ecs::system::{RunSystemError, RunSystemOnce},
		reflect::TypePath,
		transform::components::Transform,
	};
	use common::test_tools::utils::{SingleThreadedApp, new_handle};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell(Dir3);

	impl From<&_Cell> for Dir3 {
		fn from(value: &_Cell) -> Self {
			value.0
		}
	}

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: u8 = 4;
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<Map<_Cell>>>();

		app
	}

	fn add_map(app: &mut App, cells: Vec<Vec<_Cell>>) -> Handle<Map<_Cell>> {
		let handle = new_handle::<Map<_Cell>>();
		app.world_mut()
			.resource_mut::<Assets<Map<_Cell>>>()
			.insert(&handle.clone(), Map(cells));
		handle
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
	fn remove_level_load_command() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		let cmd = app.world().get_resource::<LoadLevel<Map<_Cell>>>();

		assert_eq!(None, cmd);
		Ok(())
	}

	#[test]
	fn pass_transform() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([(
					(0, 0),
					(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z))
				)]),
				extra: (),
				context: get_context::<_Cell>(1, 1),
			}),
			result
		);
		Ok(())
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_x() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(Transform::from_xyz(-2., 0., 0.), _Cell(Dir3::NEG_Z))
					),
					(
						(1, 0),
						(Transform::from_xyz(2., 0., 0.), _Cell(Dir3::NEG_Z))
					)
				]),
				extra: (),
				context: get_context::<_Cell>(2, 1),
			}),
			result
		);
		Ok(())
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_z() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Dir3::NEG_Z)], vec![_Cell(Dir3::NEG_Z)]],
		);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(Transform::from_xyz(0., 0., -2.), _Cell(Dir3::NEG_Z))
					),
					(
						(0, 1),
						(Transform::from_xyz(0., 0., 2.), _Cell(Dir3::NEG_Z))
					)
				]),
				extra: (),
				context: get_context::<_Cell>(1, 2),
			}),
			result
		);
		Ok(())
	}

	#[test]
	fn add_scene_handle_with_transform_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let direction = Dir3::new(Vec3::new(2., 3., 5.)).unwrap();
		let map_handle = add_map(&mut app, vec![vec![_Cell(direction)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([(
					(0, 0),
					(
						Transform::from_xyz(0., 0., 0.).looking_to(direction, Vec3::Y),
						_Cell(direction)
					)
				),]),
				extra: (),
				context: get_context::<_Cell>(1, 1),
			}),
			result
		);
		Ok(())
	}

	#[test]
	fn center_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(
			&mut app,
			vec![
				vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)],
				vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)],
				vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)],
			],
		);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(Transform::from_xyz(-4., 0., -4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(1, 0),
						(Transform::from_xyz(0., 0., -4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(2, 0),
						(Transform::from_xyz(4., 0., -4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(0, 1),
						(Transform::from_xyz(-4., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(1, 1),
						(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(2, 1),
						(Transform::from_xyz(4., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(0, 2),
						(Transform::from_xyz(-4., 0., 4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(1, 2),
						(Transform::from_xyz(0., 0., 4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(2, 2),
						(Transform::from_xyz(4., 0., 4.), _Cell(Dir3::NEG_Z)),
					),
				]),
				extra: (),
				context: get_context::<_Cell>(3, 3),
			}),
			result
		);
		Ok(())
	}

	#[test]
	fn center_map_with_different_row_lengths() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_handle = add_map(
			&mut app,
			vec![
				vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)],
				vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)],
				vec![_Cell(Dir3::NEG_Z)],
			],
		);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		let result = app.world_mut().run_system_once(LoadLevel::<_Cell>::graph)?;

		assert_eq!(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(Transform::from_xyz(-4., 0., -4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(1, 0),
						(Transform::from_xyz(0., 0., -4.), _Cell(Dir3::NEG_Z)),
					),
					(
						(0, 1),
						(Transform::from_xyz(-4., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(1, 1),
						(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(2, 1),
						(Transform::from_xyz(4., 0., 0.), _Cell(Dir3::NEG_Z)),
					),
					(
						(0, 2),
						(Transform::from_xyz(-4., 0., 4.), _Cell(Dir3::NEG_Z)),
					),
				]),
				extra: (),
				context: get_context::<_Cell>(3, 3),
			}),
			result
		);
		Ok(())
	}
}
