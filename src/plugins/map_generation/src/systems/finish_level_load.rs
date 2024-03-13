use crate::{components::LoadLevelCommand, traits::MapAsset};
use bevy::{
	asset::{Asset, Assets},
	ecs::system::{Commands, Res, Resource},
	math::{primitives::Direction3d, Vec3},
	scene::{Scene, SceneBundle},
	transform::components::Transform,
	utils::default,
};
use common::traits::{iteration::KeyValue, load_asset::LoadAsset};

pub(crate) fn finish_level_load<
	TAssetLoader: LoadAsset<Scene> + Resource,
	TMap: MapAsset<TCell> + Asset,
	TCell: KeyValue<Option<(Direction3d, String)>>,
>(
	mut commands: Commands,
	asset_loader: Res<TAssetLoader>,
	map: Res<Assets<TMap>>,
	load_level_cmd: Option<Res<LoadLevelCommand<TMap>>>,
) {
	let Some(cmd) = load_level_cmd else {
		return;
	};
	let Some(map) = map.get(&cmd.0) else {
		return;
	};

	let mut position = Vec3::ZERO;

	for cell_line in map.cells() {
		position.x = 0.;
		for cell in cell_line {
			let Some((direction, path)) = cell.get_value() else {
				position.x += TMap::CELL_DISTANCE;
				continue;
			};
			let direction = Vec3::from(direction);
			let scene = asset_loader.load_asset(path.clone());
			let transform = Transform::from_translation(position).looking_to(direction, Vec3::Y);
			commands.spawn(SceneBundle {
				scene,
				transform,
				..default()
			});
			position.x += TMap::CELL_DISTANCE;
		}
		position.z -= TMap::CELL_DISTANCE;
	}

	commands.remove_resource::<LoadLevelCommand<TMap>>();
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, AssetPath, Handle},
		reflect::TypePath,
		transform::components::Transform,
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::collections::HashMap;

	#[derive(Resource)]
	struct _AssetLoader<TAsset: Asset>(HashMap<String, Handle<TAsset>>);

	impl<TAsset: Asset> Default for _AssetLoader<TAsset> {
		fn default() -> Self {
			Self(Default::default())
		}
	}

	impl<TAsset: Asset> LoadAsset<TAsset> for _AssetLoader<TAsset> {
		fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<TAsset> {
			let path: AssetPath = path.into();
			self.0
				.iter()
				.find_map(|(key, value)| match AssetPath::from(key) == path {
					true => Some(value.clone()),
					false => None,
				})
				.unwrap_or(Handle::default())
		}
	}

	#[derive(Clone, Debug, PartialEq)]
	struct _Cell(Option<(Direction3d, String)>);

	impl KeyValue<Option<(Direction3d, String)>> for _Cell {
		fn get_value(self) -> Option<(Direction3d, String)> {
			self.0
		}
	}

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Map(Vec<Vec<_Cell>>);

	impl MapAsset<_Cell> for _Map {
		const CELL_DISTANCE: f32 = 4.;

		fn cells(&self) -> Vec<Vec<_Cell>> {
			self.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(
			Update,
			finish_level_load::<_AssetLoader<Scene>, _Map, _Cell>,
		);
		app.init_resource::<_AssetLoader<Scene>>();
		app.init_resource::<Assets<_Map>>();

		app
	}

	fn new_handle<TAsset: Asset>() -> Handle<TAsset> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn add_map(app: &mut App, cells: Vec<Vec<_Cell>>) -> Handle<_Map> {
		let handle = new_handle::<_Map>();
		app.world
			.resource_mut::<Assets<_Map>>()
			.insert(handle.clone(), _Map(cells));
		handle
	}

	fn add_scene(app: &mut App, path: &str) -> Handle<Scene> {
		let scene_handle = new_handle::<Scene>();
		app.world
			.resource_mut::<_AssetLoader<Scene>>()
			.0
			.insert(path.to_owned(), scene_handle.clone());

		scene_handle
	}

	#[test]
	fn remove_level_load_command() {
		let mut app = setup();
		_ = add_scene(&mut app, "A");
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Some((Direction3d::NEG_Z, "A".to_owned())))]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let cmd = app.world.get_resource::<LoadLevelCommand<_Map>>();

		assert_eq!(None, cmd);
	}

	#[test]
	fn add_scene_handle() {
		let mut app = setup();
		let scene_handle = add_scene(&mut app, "A");
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Some((Direction3d::NEG_Z, "A".to_owned())))]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scene = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Handle<Scene>>());

		assert_eq!(Some(&scene_handle), scene);
	}

	#[test]
	fn add_scene_handle_with_transform() {
		let mut app = setup();
		let scene_handle = add_scene(&mut app, "A");
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Some((Direction3d::NEG_Z, "A".to_owned())))]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scene_and_transform = app.world.iter_entities().find_map(|e| {
			e.get::<Handle<Scene>>()
				.and_then(|s| Some((s, e.get::<Transform>()?)))
		});

		assert_eq!(
			Some((&scene_handle, &Transform::from_xyz(0., 0., 0.))),
			scene_and_transform
		);
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_x() {
		let mut app = setup();
		let scene_handle_a = add_scene(&mut app, "A");
		let scene_handle_b = add_scene(&mut app, "B");
		let map_handle = add_map(
			&mut app,
			vec![vec![
				_Cell(Some((Direction3d::NEG_Z, "A".to_owned()))),
				_Cell(Some((Direction3d::NEG_Z, "B".to_owned()))),
			]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scenes_and_transforms: Vec<_> = app
			.world
			.iter_entities()
			.filter_map(|e| {
				e.get::<Handle<Scene>>()
					.and_then(|s| Some((s, e.get::<Transform>()?)))
			})
			.collect();

		assert_eq!(
			vec![
				(&scene_handle_a.clone(), &Transform::from_xyz(0., 0., 0.)),
				(&scene_handle_b.clone(), &Transform::from_xyz(4., 0., 0.))
			],
			scenes_and_transforms
		);
	}

	#[test]
	fn add_scene_handle_with_transform_with_distance_on_z() {
		let mut app = setup();
		let scene_handle_a = add_scene(&mut app, "A");
		let scene_handle_b = add_scene(&mut app, "B");
		let map_handle = add_map(
			&mut app,
			vec![
				vec![_Cell(Some((Direction3d::NEG_Z, "A".to_owned())))],
				vec![_Cell(Some((Direction3d::NEG_Z, "B".to_owned())))],
			],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scenes_and_transforms: Vec<_> = app
			.world
			.iter_entities()
			.filter_map(|e| {
				e.get::<Handle<Scene>>()
					.and_then(|s| Some((s, e.get::<Transform>()?)))
			})
			.collect();

		assert_eq!(
			vec![
				(&scene_handle_a.clone(), &Transform::from_xyz(0., 0., 0.)),
				(&scene_handle_b.clone(), &Transform::from_xyz(0., 0., -4.))
			],
			scenes_and_transforms
		);
	}

	#[test]
	fn add_scene_handle_with_transform_direction() {
		let mut app = setup();
		let scene_handle = add_scene(&mut app, "A");
		let direction = Direction3d::new(Vec3::new(1., 2., 3.)).unwrap();
		let map_handle = add_map(
			&mut app,
			vec![vec![_Cell(Some((direction, "A".to_owned())))]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scene_and_transform = app.world.iter_entities().find_map(|e| {
			e.get::<Handle<Scene>>()
				.and_then(|s| Some((s, e.get::<Transform>()?)))
		});

		assert_eq!(
			Some((
				&scene_handle,
				&Transform::from_xyz(0., 0., 0.).looking_to(Vec3::from(direction), Vec3::Y)
			)),
			scene_and_transform
		);
	}

	#[test]
	fn empty_cells_do_not_mess_with_position() {
		let mut app = setup();
		let scene_handle = add_scene(&mut app, "A");
		let map_handle = add_map(
			&mut app,
			vec![vec![
				_Cell(None),
				_Cell(Some((Direction3d::Z, "A".to_owned()))),
			]],
		);
		app.world.insert_resource(LoadLevelCommand(map_handle));

		app.update();

		let scene_and_transform = app.world.iter_entities().find_map(|e| {
			e.get::<Handle<Scene>>()
				.and_then(|s| Some((s, e.get::<Transform>()?)))
		});

		assert_eq!(
			Some((
				&scene_handle,
				&Transform::from_xyz(4., 0., 0.).looking_to(Vec3::Z, Vec3::Y)
			)),
			scene_and_transform
		);
	}
}
