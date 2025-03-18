use crate::{
	grid_graph::GridGraph,
	half_offset_grid::HalfOffsetGrid,
	map::Map,
	traits::SourcePath,
};
use bevy::prelude::*;
use common::{
	tools::handle::default_handle,
	traits::{load_asset::LoadAsset, thread_safe::ThreadSafe},
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct CurrentLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) map: Handle<Map<TCell>>,
	pub(crate) graph: Option<GridGraph>,
	pub(crate) half_offset_grid: Option<HalfOffsetGrid>,
}

impl<TCell> CurrentLevel<TCell>
where
	TCell: SourcePath + TypePath + ThreadSafe,
{
	const NO_MAP: Handle<Map<TCell>> = default_handle();

	pub(crate) fn load(
		commands: Commands,
		map_loader: ResMut<AssetServer>,
		level: Option<Res<Self>>,
	) {
		Self::load_internal(commands, map_loader, level);
	}

	fn load_internal<TLoadMap>(
		mut commands: Commands,
		mut map_loader: ResMut<TLoadMap>,
		level: Option<Res<Self>>,
	) where
		TLoadMap: LoadAsset + Resource,
	{
		if matches!(level, Some(level) if level.map != Self::NO_MAP) {
			return;
		}

		commands.insert_resource(Self {
			map: map_loader.load_asset(TCell::source_path()),
			..default()
		});
	}
}

impl<TCell> Default for CurrentLevel<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn default() -> Self {
		Self {
			map: default(),
			graph: default(),
			half_offset_grid: default(),
		}
	}
}

#[cfg(test)]
mod test_load {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{
			load_asset::{LoadAsset, Path},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

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
		app.add_systems(Update, CurrentLevel::<_Cell>::load_internal::<_LoadMap>);

		app
	}

	#[test]
	fn insert_level() {
		let map = new_handle();
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(map.clone());
		}));

		app.update();

		assert_eq!(
			Some(&CurrentLevel { map, ..default() }),
			app.world().get_resource::<CurrentLevel<_Cell>>()
		);
	}

	#[test]
	fn do_nothing_if_level_already_loaded() {
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset::<Map<_Cell>, Path>()
				.never()
				.return_const(new_handle());
		}));
		app.insert_resource(CurrentLevel::<_Cell> {
			map: new_handle(),
			..default()
		});

		app.update();
	}

	#[test]
	fn update_level_if_map_is_default() {
		let map = new_handle();
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(map.clone());
		}));
		app.insert_resource(CurrentLevel::<_Cell> {
			map: default(),
			..default()
		});

		app.update();

		assert_eq!(
			Some(&CurrentLevel { map, ..default() }),
			app.world().get_resource::<CurrentLevel<_Cell>>()
		);
	}
}
