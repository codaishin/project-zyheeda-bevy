use crate::{
	grid_graph::{GridGraph, Obstacles},
	traits::{GridCellDistanceDefinition, is_walkable::IsWalkable},
};
use bevy::prelude::*;
use common::traits::load_asset::{LoadAsset, Path};

#[derive(Component, Debug, PartialEq, Default)]
#[require(Name(Self::name), Transform, Visibility)]
pub struct Level {
	graph: GridGraph,
}

impl Level {
	fn name() -> &'static str {
		"Level"
	}

	pub(crate) fn spawn<TCell>(
		graph: In<Option<GridGraph<(Transform, TCell), ()>>>,
		commands: Commands,
		load_asset: ResMut<AssetServer>,
		level_cache: Local<Option<Entity>>,
		levels: Query<&mut Level>,
	) where
		TCell: IsWalkable + GridCellDistanceDefinition,
		for<'a> Path: TryFrom<&'a TCell>,
	{
		spawn(graph, commands, load_asset, level_cache, levels);
	}
}

impl From<&Level> for GridGraph {
	fn from(value: &Level) -> Self {
		value.graph.clone()
	}
}

pub(crate) fn spawn<TCell, TAsset>(
	In(graph): In<Option<GridGraph<(Transform, TCell), ()>>>,
	mut commands: Commands,
	mut load_asset: ResMut<TAsset>,
	mut level_cache: Local<Option<Entity>>,
	mut levels: Query<&mut Level>,
) where
	TCell: IsWalkable + GridCellDistanceDefinition,
	TAsset: LoadAsset + Resource,
	for<'a> Path: TryFrom<&'a TCell>,
{
	let Some(graph) = graph else {
		return;
	};

	if graph.nodes.is_empty() {
		return;
	}

	let mut lvl_entity = match *level_cache {
		Some(level) => get_or_new!(commands, level),
		None => commands.spawn(Level::default()),
	};
	let lvl_id = lvl_entity.id();
	let graph = apply_graph!(graph, lvl_entity, load_asset);
	*level_cache = Some(lvl_id);
	update_level_graph!(levels, lvl_entity, lvl_id, graph, TCell);
}

macro_rules! apply_graph {
	($graph:expr, $level:expr, $load_asset:expr) => {{
		let mut new_graph = GridGraph::<Vec3, Obstacles> {
			context: $graph.context,
			..default()
		};

		for (key, (transform, cell)) in $graph.nodes {
			new_graph.nodes.insert(key, transform.translation);
			if !cell.is_walkable() {
				new_graph.extra.obstacles.insert(key);
			}

			if let Ok(path) = Path::try_from(&cell) {
				let scene = $load_asset.load_asset(path);
				$level.with_child((SceneRoot(scene), transform));
			}
		}

		new_graph
	}};
}
use apply_graph;

macro_rules! get_or_new {
	($commands:expr, $entity:expr) => {
		match $commands.get_entity($entity) {
			Some(level) => level,
			None => $commands.spawn(Level::default()),
		}
	};
}
use get_or_new;

macro_rules! update_level_graph {
	($levels:expr, $level:expr, $level_id:expr, $graph:expr, $cell_ty:ty) => {
		match $levels.get_mut($level_id) {
			Ok(mut level) => {
				*level = Level { graph: $graph };
			}
			Err(_) => {
				$level.insert(Level { graph: $graph });
			}
		}
	};
}
use update_level_graph;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::grid_graph::grid_context::{GridContext, GridDefinition, GridDefinitionError};
	use bevy::asset::AssetPath;
	use common::{
		assert_count,
		get_children,
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{load_asset::Path, nested_mock::NestedMocks, thread_safe::ThreadSafe},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::{HashMap, HashSet};

	#[derive(Clone, Default)]
	struct _Cell {
		path: Option<Path>,
		is_walkable: bool,
	}

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: u8 = 0;
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

	#[derive(Clone, Default)]
	struct _CellWithDistance {
		path: Option<Path>,
		is_walkable: bool,
	}

	impl GridCellDistanceDefinition for _CellWithDistance {
		const CELL_DISTANCE: u8 = 11;
	}

	impl IsWalkable for _CellWithDistance {
		fn is_walkable(&self) -> bool {
			self.is_walkable
		}
	}

	impl TryFrom<&_CellWithDistance> for Path {
		type Error = ();

		fn try_from(value: &_CellWithDistance) -> Result<Self, Self::Error> {
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

	fn setup<TCell>(graph: Option<GridGraph<(Transform, TCell), ()>>, load_scene: _LoadScene) -> App
	where
		TCell: Clone + IsWalkable + GridCellDistanceDefinition + ThreadSafe,
		for<'a> Path: TryFrom<&'a TCell>,
	{
		let mut app = App::new().single_threaded(Update);
		let return_graph = move || graph.clone();

		app.insert_resource(load_scene);
		app.add_systems(Update, (return_graph).pipe(spawn::<TCell, _LoadScene>));

		app
	}

	#[test]
	fn spawn_scene_with_transform() {
		let scene = new_handle();
		let mut app = setup(
			Some(GridGraph {
				nodes: HashMap::from([(
					(0, 0),
					(
						Transform::from_xyz(1., 2., 3.),
						_Cell {
							path: Some(Path::from("A")),
							..default()
						},
					),
				)]),
				..default()
			}),
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
			Some(GridGraph {
				nodes: HashMap::from([(
					(0, 0),
					(
						Transform::default(),
						_Cell {
							path: Some(Path::from("A")),
							..default()
						},
					),
				)]),
				..default()
			}),
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
			Some(GridGraph {
				nodes: HashMap::from([(
					(0, 0),
					(
						Transform::default(),
						_Cell {
							path: Some(Path::from("A")),
							..default()
						},
					),
				)]),
				..default()
			}),
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
	fn store_graph_in_level() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 2,
			cell_count_z: 1,
			cell_distance: 42,
		})?;
		let mut app = setup(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(
							Transform::from_xyz(1., 2., 3.),
							_Cell {
								path: Some(Path::from("A")),
								is_walkable: true,
							},
						),
					),
					(
						(1, 0),
						(
							Transform::from_xyz(3., 4., 5.),
							_Cell {
								path: Some(Path::from("A")),
								is_walkable: false,
							},
						),
					),
				]),
				context,
				..default()
			}),
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
				graph: GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(1., 2., 3.)),
						((1, 0), Vec3::new(3., 4., 5.),),
					]),
					extra: Obstacles {
						obstacles: HashSet::from([(1, 0)]),
					},
					context,
				}
			},
			level
		);
		Ok(())
	}

	#[test]
	fn store_graph_cells_with_no_scene_path_in_level() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 2,
			cell_count_z: 1,
			cell_distance: 42,
		})?;
		let mut app = setup(
			Some(GridGraph {
				nodes: HashMap::from([
					(
						(0, 0),
						(
							Transform::from_xyz(1., 2., 3.),
							_Cell {
								path: None,
								is_walkable: true,
							},
						),
					),
					(
						(1, 0),
						(
							Transform::from_xyz(3., 4., 5.),
							_Cell {
								path: None,
								is_walkable: false,
							},
						),
					),
				]),
				context,
				..default()
			}),
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
				graph: GridGraph {
					nodes: HashMap::from([
						((0, 0), Vec3::new(1., 2., 3.)),
						((1, 0), Vec3::new(3., 4., 5.),),
					]),
					extra: Obstacles {
						obstacles: HashSet::from([(1, 0)]),
					},
					context,
				}
			},
			level
		);
		Ok(())
	}

	#[test]
	fn do_nothing_if_grid_empty() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 2,
			cell_count_z: 1,
			cell_distance: 42,
		})?;
		let mut app = setup(
			Some(GridGraph::<(Transform, _Cell), ()> {
				nodes: HashMap::from([]),
				context,
				..default()
			}),
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();

		let levels = app.world().iter_entities().filter_map(|e| e.get::<Level>());
		assert_count!(0, levels);
		Ok(())
	}

	#[test]
	fn do_nothing_if_grid_none() {
		let mut app = setup(
			None as Option<GridGraph<(Transform, _Cell), ()>>,
			_LoadScene::new().with_mock(|mock| {
				mock.expect_load_asset::<Scene, Path>()
					.return_const(new_handle());
			}),
		);

		app.update();

		let levels = app.world().iter_entities().filter_map(|e| e.get::<Level>());
		assert_count!(0, levels);
	}
}
