use crate::traits::is_walkable::IsWalkable;
use bevy::prelude::*;
use common::traits::{
	handles_map_generation::NavCell,
	iterate::Iterate,
	load_asset::{LoadAsset, Path},
};

#[derive(Component, Debug, PartialEq, Default)]
#[require(Name(Self::name), Transform, Visibility)]
pub struct Level {
	cells: Vec<NavCell>,
}

impl Level {
	fn name() -> &'static str {
		"Level"
	}

	pub(crate) fn spawn<TCell>(
		cells: In<Vec<(Transform, TCell)>>,
		commands: Commands,
		load_asset: ResMut<AssetServer>,
		level_cache: Local<Option<Entity>>,
		levels: Query<&mut Level>,
	) where
		TCell: IsWalkable,
		for<'a> Path: TryFrom<&'a TCell>,
	{
		spawn(cells, commands, load_asset, level_cache, levels);
	}
}

impl Iterate for Level {
	type TItem<'a>
		= &'a NavCell
	where
		Self: 'a;

	fn iterate(&self) -> impl Iterator<Item = &'_ NavCell> {
		self.cells.iter()
	}
}

pub(crate) fn spawn<TCell, TAsset>(
	In(cells): In<Vec<(Transform, TCell)>>,
	mut commands: Commands,
	mut load_asset: ResMut<TAsset>,
	mut level_cache: Local<Option<Entity>>,
	mut levels: Query<&mut Level>,
) where
	TCell: IsWalkable,
	TAsset: LoadAsset + Resource,
	for<'a> Path: TryFrom<&'a TCell>,
{
	let mut level = match *level_cache {
		Some(level) => get_or_new!(commands, level),
		None => commands.spawn(Level::default()),
	};
	let level_id = level.id();
	let cells = spawn_cells!(cells, level, load_asset);
	*level_cache = Some(level_id);
	update_level_cells!(levels, level, level_id, cells);
}

fn with_cell_path<TCell>((transform, cell): &(Transform, TCell)) -> Option<(Transform, Path, bool)>
where
	TCell: IsWalkable,
	for<'a> Path: TryFrom<&'a TCell>,
{
	let is_walkable = cell.is_walkable();
	Some((*transform, Path::try_from(cell).ok()?, is_walkable))
}

macro_rules! spawn_cells {
	($cells:expr, $level:expr, $load_asset:expr) => {
		$cells
			.iter()
			.filter_map(with_cell_path)
			.map(|(transform, path, is_walkable)| {
				let scene = $load_asset.load_asset(path);
				let cell = NavCell {
					translation: transform.translation,
					is_walkable,
				};
				$level.with_child((SceneRoot(scene), transform));
				cell
			})
	};
}
use spawn_cells;

macro_rules! get_or_new {
	($commands:expr, $entity:expr) => {
		match $commands.get_entity($entity) {
			Some(level) => level,
			None => $commands.spawn(Level::default()),
		}
	};
}
use get_or_new;

macro_rules! update_level_cells {
	($levels:expr, $level:expr, $level_id:expr, $cells:expr) => {
		match $levels.get_mut($level_id) {
			Ok(mut level) => {
				level.cells.extend($cells);
			}
			Err(_) => {
				let cells = $cells.collect();
				$level.insert(Level { cells });
			}
		}
	};
}
use update_level_cells;

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		assert_count,
		get_children,
		test_tools::utils::{new_handle, SingleThreadedApp},
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Clone, Default)]
	struct _Cell {
		path: Option<Path>,
		is_walkable: bool,
	}

	impl IsWalkable for _Cell {
		fn is_walkable(&self) -> bool {
			self.is_walkable
		}
	}

	impl TryFrom<&_Cell> for Path {
		type Error = ();

		fn try_from(value: &_Cell) -> Result<Self, Self::Error> {
			match &value.path {
				Some(path) => Ok(path.clone()),
				None => Err(()),
			}
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LoadScene {
		mock: Mock_LoadScene,
	}

	#[automock]
	impl LoadAsset for _LoadScene {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(cells: Vec<(Transform, _Cell)>, load_scene: _LoadScene) -> App {
		let mut app = App::new().single_threaded(Update);
		let return_cells = move || cells.clone();

		app.insert_resource(load_scene);
		app.add_systems(Update, (return_cells).pipe(spawn::<_Cell, _LoadScene>));

		app
	}

	#[test]
	fn spawn_scene_with_transform() {
		let scene = new_handle();
		let mut app = setup(
			vec![(
				Transform::from_xyz(1., 2., 3.),
				_Cell {
					path: Some(Path::from("A")),
					..default()
				},
			)],
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset()
					.times(1)
					.with(eq(Path::from("A")))
					.return_const(scene.clone());
			}),
		);

		app.update();

		let spawned = app
			.world()
			.iter_entities()
			.filter_map(|e| Some((e.get::<SceneRoot>()?, e.get::<Transform>()?)));
		let [spawned] = assert_count!(1, spawned);
		assert_eq!(
			(&SceneRoot(scene), &Transform::from_xyz(1., 2., 3.)),
			spawned
		);
	}

	#[test]
	fn spawn_scene_as_child_of_level() {
		let mut app = setup(
			vec![(
				Transform::default(),
				_Cell {
					path: Some(Path::from("A")),
					..default()
				},
			)],
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();

		let levels = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<Level>());
		let [level] = assert_count!(1, levels);
		let spawned = get_children!(app, level.id()).filter(|c| c.contains::<SceneRoot>());
		assert_count!(1, spawned);
	}

	#[test]
	fn reuse_same_level_in_subsequent_updates() {
		let mut app = setup(
			vec![(
				Transform::default(),
				_Cell {
					path: Some(Path::from("A")),
					..default()
				},
			)],
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();
		app.update();

		let levels = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<Level>());
		let [level] = assert_count!(1, levels);
		let spawned = get_children!(app, level.id()).filter(|c| c.contains::<SceneRoot>());
		assert_count!(2, spawned);
	}

	#[test]
	fn store_nav_cell_in_level() {
		let mut app = setup(
			vec![
				(
					Transform::from_xyz(1., 2., 3.),
					_Cell {
						path: Some(Path::from("A")),
						is_walkable: false,
					},
				),
				(
					Transform::from_xyz(3., 4., 5.),
					_Cell {
						path: Some(Path::from("A")),
						is_walkable: true,
					},
				),
			],
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();

		let levels = app.world().iter_entities().filter_map(|e| e.get::<Level>());
		let [level] = assert_count!(1, levels);
		assert_eq!(
			&Level {
				cells: vec![
					NavCell {
						translation: Vec3::new(1., 2., 3.),
						is_walkable: false
					},
					NavCell {
						translation: Vec3::new(3., 4., 5.),
						is_walkable: true
					}
				]
			},
			level
		);
	}

	#[test]
	fn store_cell_transform_in_level_on_update() {
		let mut app = setup(
			vec![(
				Transform::from_xyz(1., 2., 3.),
				_Cell {
					path: Some(Path::from("A")),
					..default()
				},
			)],
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();
		app.update();

		let levels = app.world().iter_entities().filter_map(|e| e.get::<Level>());
		let [level] = assert_count!(1, levels);
		assert_eq!(
			&Level {
				cells: vec![
					NavCell {
						translation: Vec3::new(1., 2., 3.),
						is_walkable: false
					},
					NavCell {
						translation: Vec3::new(1., 2., 3.),
						is_walkable: false
					}
				]
			},
			level
		);
	}
}
