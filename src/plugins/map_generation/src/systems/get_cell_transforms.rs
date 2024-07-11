use crate::{components::LoadLevelCommand, map::Map, traits::CellDistance};
use bevy::{
	asset::Assets,
	ecs::system::{Commands, Res},
	math::{Dir3, Vec3},
	reflect::TypePath,
	transform::components::Transform,
};

pub(crate) fn get_cell_transforms<TCell: CellDistance + TypePath + Sync + Send + Clone>(
	mut commands: Commands,
	maps: Res<Assets<Map<TCell>>>,
	load_level_cmd: Option<Res<LoadLevelCommand<TCell>>>,
) -> Vec<(Transform, TCell)>
where
	Dir3: From<TCell>,
{
	let Some(cells) = get_map_cells(load_level_cmd, maps) else {
		return vec![];
	};
	let Some((start_x, start_z)) = get_start_x_z(&cells, TCell::CELL_DISTANCE) else {
		return vec![];
	};

	commands.remove_resource::<LoadLevelCommand<TCell>>();

	let mut position = Vec3::new(start_x, 0., start_z);
	let mut transforms_and_cells = vec![];

	for cell_line in cells {
		for cell in cell_line {
			transforms_and_cells.push((transform(&cell, position), cell));
			position.x -= TCell::CELL_DISTANCE;
		}
		position.x = start_x;
		position.z -= TCell::CELL_DISTANCE;
	}

	transforms_and_cells
}

fn get_map_cells<TCell: TypePath + Sync + Send + Clone>(
	load_level_cmd: Option<Res<LoadLevelCommand<TCell>>>,
	maps: Res<Assets<Map<TCell>>>,
) -> Option<Vec<Vec<TCell>>> {
	let map_handle = &load_level_cmd?.0;
	let map = maps.get(map_handle)?;

	Some(map.0.clone())
}

fn transform<TCell: Clone>(cell: &TCell, position: Vec3) -> Transform
where
	Dir3: From<TCell>,
{
	let direction = Vec3::from(Dir3::from(cell.clone()));

	Transform::from_translation(position).looking_to(direction, Vec3::Y)
}

fn get_start_x_z<T>(cells: &[Vec<T>], cell_distance: f32) -> Option<(f32, f32)> {
	let max_x = cells.iter().map(|line| line.len()).max()? as f32 * cell_distance;
	let max_z = cells.len() as f32 * cell_distance;
	let start_x = (max_x - cell_distance) / 2.;
	let start_z = (max_z - cell_distance) / 2.;
	Some((start_x, start_z))
}

#[cfg(test)]
mod tests {
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

	impl From<_Cell> for Dir3 {
		fn from(value: _Cell) -> Self {
			value.0
		}
	}

	impl CellDistance for _Cell {
		const CELL_DISTANCE: f32 = 4.;
	}

	#[derive(Resource, Default)]
	struct _CellsResult(Vec<(Transform, _Cell)>);

	fn store_result(result: In<Vec<(Transform, _Cell)>>, mut commands: Commands) {
		commands.insert_resource(_CellsResult(result.0));
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, get_cell_transforms::<_Cell>.pipe(store_result));
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
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let cmd = app.world().get_resource::<LoadLevelCommand<Map<_Cell>>>();

		assert_eq!(None, cmd);
	}

	#[test]
	fn pass_transform() {
		let mut app = setup();
		let map_handle = add_map(&mut app, vec![vec![_Cell(Dir3::NEG_Z)]]);
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
		let direction = Dir3::new(Vec3::new(1., 2., 3.)).unwrap();
		let map_handle = add_map(&mut app, vec![vec![_Cell(direction)]]);
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
		app.world_mut()
			.insert_resource(LoadLevelCommand(map_handle));

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
