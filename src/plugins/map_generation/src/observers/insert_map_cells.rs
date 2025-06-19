use crate::components::map::{MapAssetCells, MapAssetPath};
use bevy::prelude::*;
use common::traits::{load_asset::LoadAsset, thread_safe::ThreadSafe, try_insert_on::TryInsertOn};

impl<TCell> MapAssetPath<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn insert_map_cells(
		trigger: Trigger<OnInsert, Self>,
		maps: Query<&Self>,
		asset_server: ResMut<AssetServer>,
		commands: Commands,
	) {
		Self::insert_map_cells_internal(trigger, maps, asset_server, commands);
	}

	fn insert_map_cells_internal<TAssetLoader>(
		trigger: Trigger<OnInsert, Self>,
		maps: Query<&Self>,
		mut asset_loader: ResMut<TAssetLoader>,
		mut commands: Commands,
	) where
		TAssetLoader: Resource + LoadAsset,
	{
		let entity = trigger.target();
		let Ok(map) = maps.get(entity) else {
			return;
		};
		let cells = asset_loader.load_asset(map.asset_path.clone());

		commands.try_insert_on(entity, MapAssetCells::<TCell>::from(cells));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::map_cells::MapCells;
	use bevy::asset::AssetPath;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(TypePath, Debug, PartialEq, Clone)]
	struct _Cell;

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
		app.add_observer(MapAssetPath::<_Cell>::insert_map_cells_internal::<_LoadMap>);

		app
	}

	#[test]
	fn set_cells() {
		let cells = new_handle();
		let map = MapAssetPath::<_Cell>::from(Path::from("aaa/bbb/ccc.file_format"));
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(cells.clone());
		}));
		let entity = app.world_mut().spawn(map);

		assert_eq!(
			Some(&MapAssetCells::from(cells)),
			entity.get::<MapAssetCells<_Cell>>()
		);
	}

	#[test]
	fn act_again_when_overridden() {
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset::<MapCells<_Cell>, Path>()
				.times(2)
				.return_const(new_handle());
		}));

		app.world_mut()
			.spawn(MapAssetPath::<_Cell>::from(Path::from("")))
			.insert(MapAssetPath::<_Cell>::from(Path::from("")));
	}
}
