use crate::{
	map::Map,
	traits::{GridCellDistanceDefinition, SourcePath},
};
use bevy::prelude::*;
use common::traits::{load_asset::LoadAsset, thread_safe::ThreadSafe};

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

	pub(crate) fn cell_transforms(
		mut commands: Commands,
		maps: Res<Assets<Map<TCell>>>,
		load_level_cmd: Option<Res<LoadLevel<TCell>>>,
	) -> Vec<(Transform, TCell)>
	where
		TCell: GridCellDistanceDefinition + Clone,
		for<'a> Dir3: From<&'a TCell>,
	{
		let Some(cells) = get_map_cells(load_level_cmd, maps) else {
			return vec![];
		};
		let Some((start_x, start_z)) = get_start_x_z(&cells, TCell::CELL_DISTANCE) else {
			return vec![];
		};

		commands.remove_resource::<LoadLevel<TCell>>();

		let mut position = Vec3::new(start_x, 0., start_z);
		let mut transforms_and_cells = vec![];

		for cell_line in cells {
			for cell in cell_line {
				transforms_and_cells.push((transform(&cell, position), cell));
				position.x -= TCell::CELL_DISTANCE as f32;
			}
			position.x = start_x;
			position.z -= TCell::CELL_DISTANCE as f32;
		}

		transforms_and_cells
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

fn get_start_x_z<T>(cells: &[Vec<T>], cell_distance: u8) -> Option<(f32, f32)> {
	let count_x = cells.iter().map(|line| line.len()).max()?;
	let count_z = cells.len();
	let start_x = get_start(count_x, cell_distance);
	let start_z = get_start(count_z, cell_distance);
	Some((start_x, start_z))
}

fn get_start(count: usize, cell_distance: u8) -> f32 {
	((count as u8 * cell_distance) - cell_distance) as f32 / 2.
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
mod test_transforms {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		ecs::system::{In, IntoSystem, Resource},
		reflect::TypePath,
		transform::components::Transform,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use uuid::Uuid;

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

	#[derive(Resource, Default)]
	struct _CellsResult(Vec<(Transform, _Cell)>);

	fn store_result(result: In<Vec<(Transform, _Cell)>>, mut commands: Commands) {
		commands.insert_resource(_CellsResult(result.0));
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			LoadLevel::<_Cell>::cell_transforms.pipe(store_result),
		);
		app.init_resource::<Assets<Map<_Cell>>>();
		app.init_resource::<_CellsResult>();

		app
	}

	fn new_handle<TAsset: Asset>() -> Handle<TAsset> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn add_map(app: &mut App, cells: Vec<Vec<_Cell>>) -> Handle<Map<_Cell>> {
		let handle = new_handle::<Map<_Cell>>();
		app.world_mut()
			.resource_mut::<Assets<Map<_Cell>>>()
			.insert(&handle.clone(), Map(cells));
		handle
	}

	#[test]
	fn remove_level_load_command() {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.update();

		let cmd = app.world().get_resource::<LoadLevel<Map<_Cell>>>();

		assert_eq!(None, cmd);
	}

	#[test]
	fn pass_transform() {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z))],
			result.0
		);
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_x() {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z), _Cell(Dir3::NEG_Z)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![
				(Transform::from_xyz(2., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(-2., 0., 0.), _Cell(Dir3::NEG_Z))
			],
			result.0
		);
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_z() {
		let mut app = setup();
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Dir3::NEG_Z)], vec![_Cell(Dir3::NEG_Z)]],
		);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![
				(Transform::from_xyz(0., 0., 2.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., -2.), _Cell(Dir3::NEG_Z))
			],
			result.0
		);
	}

	#[test]
	fn add_scene_handle_with_transform_direction() {
		let mut app = setup();
		let direction = Dir3::new(Vec3::new(2., 3., 5.)).unwrap();
		let map_handle = add_map(&mut app, vec![vec![_Cell(direction)]]);
		app.world_mut().insert_resource(LoadLevel(map_handle));

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![(
				Transform::from_xyz(0., 0., 0.).looking_to(direction, Vec3::Y),
				_Cell(direction)
			),],
			result.0
		);
	}

	#[test]
	fn center_map() {
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

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![
				(Transform::from_xyz(4., 0., 4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., 4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(-4., 0., 4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(4., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(-4., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(4., 0., -4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., -4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(-4., 0., -4.), _Cell(Dir3::NEG_Z)),
			],
			result.0
		);
	}

	#[test]
	fn center_map_with_uneven_row_lengths() {
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

		app.update();

		let result = app.world().resource::<_CellsResult>();

		assert_eq!(
			vec![
				(Transform::from_xyz(4., 0., 4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., 4.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(4., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(0., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(-4., 0., 0.), _Cell(Dir3::NEG_Z)),
				(Transform::from_xyz(4., 0., -4.), _Cell(Dir3::NEG_Z)),
			],
			result.0
		);
	}
}
