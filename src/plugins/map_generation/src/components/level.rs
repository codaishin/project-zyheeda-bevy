use bevy::prelude::*;
use common::traits::load_asset::{LoadAsset, Path};

#[derive(Component, Debug, PartialEq)]
#[require(Name(Self::name), Transform, Visibility)]
pub(crate) struct Level;

impl Level {
	fn name() -> &'static str {
		"Level"
	}

	pub(crate) fn spawn<TCell>(
		cells: In<Vec<(Transform, TCell)>>,
		commands: Commands,
		load_asset: ResMut<AssetServer>,
		level_cache: Local<Option<Entity>>,
	) where
		for<'a> Path: TryFrom<&'a TCell>,
	{
		spawn(cells, commands, load_asset, level_cache);
	}
}

pub(crate) fn spawn<TCell, TAsset>(
	cells: In<Vec<(Transform, TCell)>>,
	mut commands: Commands,
	mut load_asset: ResMut<TAsset>,
	mut level_cache: Local<Option<Entity>>,
) where
	TAsset: LoadAsset + Resource,
	for<'a> Path: TryFrom<&'a TCell>,
{
	let mut level = match *level_cache {
		Some(level) => get_or_new!(commands, level),
		None => commands.spawn(Level),
	};

	*level_cache = Some(level.id());

	for (transform, path) in cells.0.iter().filter_map(with_cell_path) {
		let scene = load_asset.load_asset(path);
		level.with_child((SceneRoot(scene), transform));
	}
}

macro_rules! get_or_new {
	($commands:expr, $entity:expr) => {
		match $commands.get_entity($entity) {
			Some(level) => level,
			None => $commands.spawn(Level),
		}
	};
}

use get_or_new;

fn with_cell_path<TCell>((transform, cell): &(Transform, TCell)) -> Option<(Transform, Path)>
where
	for<'a> Path: TryFrom<&'a TCell>,
{
	Some((*transform, Path::try_from(cell).ok()?))
}

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

	#[derive(Clone)]
	struct _Cell(Option<Path>);

	impl TryFrom<&_Cell> for Path {
		type Error = ();

		fn try_from(value: &_Cell) -> Result<Self, Self::Error> {
			match &value.0 {
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
				_Cell(Some(Path::from("A"))),
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
			vec![(Transform::default(), _Cell(Some(Path::from("A"))))],
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
			vec![(Transform::default(), _Cell(Some(Path::from("A"))))],
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
}
